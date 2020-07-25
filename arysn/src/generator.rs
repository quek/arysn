use anyhow::Result;
use belongs_to::{make_belongs_to, BelongsTo};
use config::Config;
use has_many::{make_has_many, HasMany};
use has_one::{make_has_one, HasOne};
use inflector::Inflector;
use log::debug;
use order::order_part;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tokio::runtime::Runtime;
use tokio_postgres::{Client, NoTls};

mod belongs_to;
pub mod config;
mod enums;
mod has_many;
mod has_one;
mod order;

pub fn define_ar(config: &Config) -> Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();

    let (output_plain, output_impl): (TokenStream, TokenStream) = define_ar_impl(config).unwrap();
    let path = &config.path;
    {
        println!("path {}", &path);
        let mut writer = std::io::BufWriter::new(std::fs::File::create(path)?);
        writeln!(writer, "{}", &output_plain.to_string())?;
    }
    Command::new("rustfmt")
        .arg("--edition")
        .arg("2018")
        .arg(path)
        .output()?;

    let path = &config.path.replace(".rs", "_impl.rs");
    {
        let mut writer = std::io::BufWriter::new(std::fs::File::create(path)?);
        writeln!(writer, "{}", &output_impl.to_string())?;
    }
    Command::new("rustfmt")
        .arg("--edition")
        .arg("2018")
        .arg(path)
        .output()?;
    Ok(())
}

async fn columns(table_name: &String, client: &Client) -> Result<Vec<Column>> {
    let primary_key: Vec<String> = primary_key(table_name, client).await?;
    let sql = format!(
        r"
SELECT column_name, is_nullable, data_type, column_default, udt_name
FROM
  information_schema.columns
WHERE
  table_name = '{}'
ORDER BY ordinal_position
",
        table_name
    );
    let rows = client.query(sql.as_str(), &[]).await?;
    let result: Vec<Column> = rows
        .iter()
        .map(|row| {
            let name: String = row.get(0);
            let is_nullable: &str = row.get(1);
            let is_nullable: bool = is_nullable == "YES";
            let data_type: String = row.get(2);
            let column_default: Option<String> = row.get(3);
            let udt_name: String = row.get(4);
            let (rust_type, nullable_rust_type) = compute_type(&data_type, is_nullable, &udt_name);
            let rust_type_for_new = if column_default.is_some() && !is_nullable {
                quote! { Option<#rust_type> }
            } else {
                nullable_rust_type.clone()
            };
            let is_primary_key = primary_key.iter().any(|x| x == &name);
            Column {
                name,
                is_nullable,
                data_type,
                column_default,
                rust_type,
                rust_type_for_new,
                nullable_rust_type,
                is_primary_key,
                udt_name,
            }
        })
        .collect();
    Ok(result)
}

fn define_ar_impl(config: &Config) -> Result<(TokenStream, TokenStream)> {
    let mut rt = Runtime::new()?;

    rt.block_on(async {
        let client = connect().await?;

        let table_name: String = config.table_name.to_string();
        let columns: Vec<Column> = columns(&table_name, &client).await?;

        let mut column_names = Vec::<Ident>::new();
        let mut rust_types = Vec::<TokenStream>::new();
        let mut rust_types_for_new = Vec::<TokenStream>::new();
        let mut nullable_rust_types = Vec::<TokenStream>::new();
        for column in columns.iter() {
            column_names.push(format_ident!("{}", &column.name));
            rust_types.push(column.rust_type.clone());
            rust_types_for_new.push(column.rust_type_for_new.clone());
            nullable_rust_types.push(column.nullable_rust_type.clone());
        }
        let column_index = 0..columns.len();

        let module_name: Ident = format_ident!(
            "{}",
            PathBuf::from(&config.path)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
        );
        let struct_ident: Ident = format_ident!("{}", &config.struct_name);
        let new_struct_ident: Ident = format_ident!("{}New", struct_ident);
        let builder_ident: Ident = format_ident!("{}Builder", struct_ident);
        let builder_columns: Vec<Ident> = columns
            .iter()
            .map(|column| format_ident!("{}_{}", &builder_ident, &column.name))
            .collect();

        let fn_delete: TokenStream = make_fn_delete(&table_name, &columns);
        let fn_insert: TokenStream = make_fn_insert(&struct_ident, &table_name, &columns);
        let fn_update: TokenStream = make_fn_update(&table_name, &columns);

        let enums = enums::definitions(&columns, &client).await?;
        let use_to_sql_from_sql = if enums.is_empty() {
            quote!()
        } else {
            quote!(
                #[cfg(target_arch = "x86_64")]
                use postgres_types::{FromSql, ToSql};
            )
        };

        let HasMany {
            has_many_use_plain,
            has_many_use_impl,
            has_many_field,
            has_many_init,
            has_many_builder_field,
            has_many_builder_impl,
            has_many_filters_impl,
            has_many_join,
            has_many_preload,
        } = make_has_many(config, &builder_ident);

        let HasOne {
            has_one_use_plain,
            has_one_use_impl,
            has_one_field,
            has_one_init,
            has_one_builder_field,
            has_one_builder_impl,
            has_one_filters_impl,
            has_one_join,
            has_one_preload,
        } = make_has_one(config, &builder_ident);

        let BelongsTo {
            belongs_to_use_plain,
            belongs_to_use_impl,
            belongs_to_field,
            belongs_to_init,
            belongs_to_builder_field,
            belongs_to_builder_impl,
            belongs_to_filters_impl,
            belongs_to_join,
            belongs_to_preload,
        } = make_belongs_to(config, &builder_ident, &columns);
        let use_plain = uniq_use(has_many_use_plain, has_one_use_plain, belongs_to_use_plain);
        let use_impl = uniq_use(has_many_use_impl, has_one_use_impl, belongs_to_use_impl);

        let output_plain = quote! {
            use serde::{Deserialize, Serialize};
            #use_to_sql_from_sql
            #(#use_plain)*

            #(#enums)*

            #[derive(Clone, Debug, Deserialize, Serialize)]
            pub struct #struct_ident {
                #(pub #column_names: #nullable_rust_types,)*
                #(#has_many_field)*
                #(#has_one_field)*
                #(#belongs_to_field)*
            }

            #[derive(Clone, Debug, Deserialize, Serialize)]
            pub struct #new_struct_ident {
                #(pub #column_names: #rust_types_for_new,)*
            }
        };

        let order_part: TokenStream =
            order_part(&struct_ident, &builder_ident, &columns, &table_name);
        let output_impl = quote! {
            use arysn::prelude::*;
            use async_recursion::async_recursion;
            use super::#module_name::*;
            #(#use_impl)*

            impl #struct_ident {
                pub fn select() -> #builder_ident {
                    #builder_ident {
                        from: #table_name.to_string(),
                        ..#builder_ident::default()
                    }
                }
                #fn_delete
                #fn_update
            }

            impl #new_struct_ident {
                #fn_insert
            }

            impl From<tokio_postgres::row::Row> for #struct_ident {
                fn from(row: tokio_postgres::row::Row) -> Self {
                    Self {
                        #(
                            #column_names: row.get(#column_index),
                        )*
                        #(#has_many_init)*
                        #(#has_one_init)*
                        #(#belongs_to_init)*
                    }
                }
            }

            #[derive(Clone, Debug, Default)]
            pub struct #builder_ident {
                pub from: String,
                pub table_name_as: Option<String>,
                pub filters: Vec<Filter>,
                pub preload: bool,
                pub orders: Vec<OrderItem>,
                pub limit: Option<usize>,
                pub offset: Option<usize>,
                #(#has_many_builder_field)*
                #(#has_one_builder_field)*
                #(#belongs_to_builder_field)*
            }

            impl #builder_ident {
                #(pub fn #column_names(&self) -> #builder_columns {
                    #builder_columns {
                        builder: self.clone()
                    }
                })*
                #(#has_many_builder_impl)*
                #(#has_one_builder_impl)*
                #(#belongs_to_builder_impl)*

                pub fn r#as(&self, name: String) -> Self {
                    Self  {
                        table_name_as: Some(name),
                        ..self.clone()
                    }
                }

                pub fn limit(&self, value: usize) -> Self {
                    Self {
                        limit: Some(value),
                        ..self.clone()
                    }
                }

                pub fn offset(&self, value: usize) -> Self {
                    Self {
                        offset: Some(value),
                        ..self.clone()
                    }
                }

                pub fn preload(&self) -> Self {
                    Self {
                        preload: true,
                        ..self.clone()
                    }
                }

                pub async fn count(&self, client: &tokio_postgres::Client) ->
                    anyhow::Result<i64> {
                    let (sql, params) = BuilderTrait::count(self);
                    let row = client .query_one(sql.as_str(), &params).await?;
                    let x: i64 = row.get(0);
                    Ok(x)
                }

                pub async fn first(&self, client: &tokio_postgres::Client) ->
                    anyhow::Result<#struct_ident> {
                    let params = self.select_params();
                    let row = client
                        .query_one(self.select_sql().as_str(), &params[..])
                        .await?;
                    let x: #struct_ident = #struct_ident::from(row);
                    Ok(x)
                }

                #[async_recursion]
                pub async fn load(&self, client: &tokio_postgres::Client) ->
                    anyhow::Result<Vec<#struct_ident>> {
                    let params = self.select_params();
                    let rows = client
                        .query(self.select_sql().as_str(), &params[..])
                        .await?;
                    let mut result: Vec<#struct_ident> = rows.into_iter()
                            .map(|row| #struct_ident::from(row)).collect();
                    #(#has_many_preload)*
                    #(#has_one_preload)*
                    #(#belongs_to_preload)*
                    Ok(result)
                }
            }

            impl BuilderTrait for #builder_ident {
                fn select(&self) -> String {
                    #table_name.to_string()
                }

                fn from(&self) -> String {
                    let mut result: Vec<String> = vec![self.from.clone()];
                    self.join(&mut result);
                    result.join(" ")
                }

                fn join(&self, join_parts: &mut Vec<String>) {
                    #(#has_many_join)*
                    #(#has_one_join)*
                    #(#belongs_to_join)*
                }

                fn filters(&self) -> Vec<&Filter> {
                    let mut result: Vec<&Filter> = self.filters.iter().collect();
                    #(#has_many_filters_impl)*
                    #(#has_one_filters_impl)*
                    #(#belongs_to_filters_impl)*
                    result
                }

                fn order(&self) -> &Vec<OrderItem> {
                    &self.orders
                }

                fn limit(&self) -> Option<usize> {
                    self.limit
                }

                fn offset(&self) -> Option<usize> {
                    self.offset
                }
            }

            #(
                #[allow(non_camel_case_types)]
                pub struct #builder_columns {
                    pub builder: #builder_ident,
                }
                impl #builder_columns {
                    pub fn eq(&self, value: #rust_types) -> #builder_ident {
                        let mut filters = self.builder.filters.clone();
                        filters.push(Filter {
                            table: self.builder.table_name_as.as_ref()
                                .unwrap_or(&#table_name.to_string()).to_string(),
                            name: stringify!(#column_names).to_string(),
                            values: vec![Box::new(value)],
                            operator: "=".to_string()
                        });
                        #builder_ident {
                            filters,
                            ..self.builder.clone()
                        }
                    }

                    pub fn gt(&self, value: #rust_types) -> #builder_ident {
                        let mut filters = self.builder.filters.clone();
                        filters.push(Filter {
                            table: self.builder.table_name_as.as_ref()
                                .unwrap_or(&#table_name.to_string()).to_string(),
                            name: stringify!(#column_names).to_string(),
                            values: vec![Box::new(value)],
                            operator: ">".to_string()
                        });
                        #builder_ident {
                            filters,
                            ..self.builder.clone()
                        }
                    }

                    pub fn lt(&self, value: #rust_types) -> #builder_ident {
                        let mut filters = self.builder.filters.clone();
                        filters.push(Filter {
                            table: self.builder.table_name_as.as_ref()
                                .unwrap_or(&#table_name.to_string()).to_string(),
                            name: stringify!(#column_names).to_string(),
                            values: vec![Box::new(value)],
                            operator: "<".to_string()
                        });
                        #builder_ident {
                            filters,
                            ..self.builder.clone()
                        }
                    }

                    pub fn gte(&self, value: #rust_types) -> #builder_ident {
                        let mut filters = self.builder.filters.clone();
                        filters.push(Filter {
                            table: self.builder.table_name_as.as_ref()
                                .unwrap_or(&#table_name.to_string()).to_string(),
                            name: stringify!(#column_names).to_string(),
                            values: vec![Box::new(value)],
                            operator: ">=".to_string()
                        });
                        #builder_ident {
                            filters,
                            ..self.builder.clone()
                        }
                    }

                    pub fn lte(&self, value: #rust_types) -> #builder_ident {
                        let mut filters = self.builder.filters.clone();
                        filters.push(Filter {
                            table: self.builder.table_name_as.as_ref()
                                .unwrap_or(&#table_name.to_string()).to_string(),
                            name: stringify!(#column_names).to_string(),
                            values: vec![Box::new(value)],
                            operator: "<=".to_string()
                        });
                        #builder_ident {
                            filters,
                            ..self.builder.clone()
                        }
                    }

                    pub fn not_eq(&self, value: #rust_types) -> #builder_ident {
                        let mut filters = self.builder.filters.clone();
                        filters.push(Filter {
                            table: self.builder.table_name_as.as_ref()
                                .unwrap_or(&#table_name.to_string()).to_string(),
                            name: stringify!(#column_names).to_string(),
                            values: vec![Box::new(value)],
                            operator: "<>".to_string()
                        });
                        #builder_ident {
                            filters,
                            ..self.builder.clone()
                        }
                    }

                    pub fn is_null(&self) -> #builder_ident {
                        let mut filters = self.builder.filters.clone();
                        filters.push(Filter {
                            table: self.builder.table_name_as.as_ref()
                                .unwrap_or(&#table_name.to_string()).to_string(),
                            name: stringify!(#column_names).to_string(),
                            values: vec![],
                            operator: "IS NULL".to_string()
                        });
                        #builder_ident {
                            filters,
                            ..self.builder.clone()
                        }
                    }

                    pub fn is_not_null(&self) -> #builder_ident {
                        let mut filters = self.builder.filters.clone();
                        filters.push(Filter {
                            table: self.builder.table_name_as.as_ref()
                                .unwrap_or(&#table_name.to_string()).to_string(),
                            name: stringify!(#column_names).to_string(),
                            values: vec![],
                            operator: "IS NOT NULL".to_string()
                        });
                        #builder_ident {
                            filters,
                            ..self.builder.clone()
                        }
                    }

                    pub fn between(&self, from: #rust_types, to: #rust_types) -> #builder_ident {
                        let mut filters = self.builder.filters.clone();
                        filters.push(Filter {
                            table: self.builder.table_name_as.as_ref()
                                .unwrap_or(&#table_name.to_string()).to_string(),
                            name: stringify!(#column_names).to_string(),
                            values: vec![Box::new(from), Box::new(to)],
                            operator: "BETWEEN".to_string()
                        });
                        #builder_ident {
                            filters,
                            ..self.builder.clone()
                        }
                    }

                    pub fn eq_any(&self, values: Vec<#rust_types>) -> #builder_ident {
                        let mut filters = self.builder.filters.clone();
                        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
                        for v in values {
                            vs.push(Box::new(v));
                        }
                        filters.push(Filter {
                            table: self.builder.table_name_as.as_ref()
                                .unwrap_or(&#table_name.to_string()).to_string(),
                            name: stringify!(#column_names).to_string(),
                            values: vs,
                            operator: "IN".to_string(),
                        });
                        #builder_ident {
                            filters,
                            ..self.builder.clone()
                        }
                    }

                    pub fn r#in(&self, values: Vec<#rust_types>) -> #builder_ident {
                        let mut filters = self.builder.filters.clone();
                        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
                        for v in values {
                            vs.push(Box::new(v));
                        }
                        filters.push(Filter {
                            table: self.builder.table_name_as.as_ref()
                                .unwrap_or(&#table_name.to_string()).to_string(),
                            name: stringify!(#column_names).to_string(),
                            values: vs,
                            operator: "IN".to_string(),
                        });
                        #builder_ident {
                            filters,
                            ..self.builder.clone()
                        }
                    }

                    pub fn not_in(&self, values: Vec<#rust_types>) -> #builder_ident {
                        let mut filters = self.builder.filters.clone();
                        let mut vs: Vec<Box<dyn ToSqlValue>> = vec![];
                        for v in values {
                            vs.push(Box::new(v));
                        }
                        filters.push(Filter {
                            table: self.builder.table_name_as.as_ref()
                                .unwrap_or(&#table_name.to_string()).to_string(),
                            name: stringify!(#column_names).to_string(),
                            values: vs,
                            operator: "NOT IN".to_string(),
                        });
                        #builder_ident {
                            filters,
                            ..self.builder.clone()
                        }
                    }
                }
            )*

            #order_part
        };
        Ok((output_plain, output_impl))
    })
}

async fn primary_key(table_name: &String, client: &Client) -> Result<Vec<String>> {
    let sql = format!(
        r"
SELECT kcu.column_name
FROM
  information_schema.table_constraints tc
    INNER JOIN information_schema.key_column_usage kcu
            ON tc.constraint_catalog = kcu.constraint_catalog
           AND tc.constraint_schema = kcu.constraint_schema
           AND tc.constraint_name = kcu.constraint_name
WHERE
      tc.table_name = '{}'
  AND tc.constraint_type = 'PRIMARY KEY'
ORDER BY kcu.ordinal_position
",
        table_name
    );
    let rows = client.query(sql.as_str(), &[]).await?;
    let result: Vec<String> = rows.iter().map(|row| row.get::<_, String>(0)).collect();
    Ok(result)
}

fn compute_type(
    data_type: &str,
    is_nullable: bool,
    udt_name: &String,
) -> (TokenStream, TokenStream) {
    let rust_type = match data_type {
        "bigint" => quote!(i64),
        "boolean" => quote!(bool),
        "character varying" => quote!(String),
        "date" => quote!(chrono::NaiveDate),
        "integer" => quote!(i32),
        "smallint" => quote!(i16),
        "text" => quote!(String),
        "timestamp with time zone" => quote!(chrono::DateTime<chrono::Local>),
        "timestamp without time zone" => quote!(chrono::NaiveDateTime),
        "USER-DEFINED" => {
            let name = format_ident!("{}", udt_name.to_title_case().replace(" ", ""));
            quote!(#name)
        }
        _ => panic!("unknown sql type: {}", data_type),
    };
    if is_nullable {
        (rust_type.clone(), quote!(Option<#rust_type>))
    } else {
        (rust_type.clone(), rust_type)
    }
}

async fn connect() -> Result<Client> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set!");
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            debug!("connection error: {}", e);
        }
    });
    if std::env::var("TRACE_SQL").map_or(false, |x| x == "1") {
        client.execute("SET log_statement = 'all'", &[]).await?;
    }
    client.execute("SET TIME ZONE 'Japan'", &[]).await?;
    Ok(client)
}

pub struct Column {
    pub name: String,
    pub is_nullable: bool,
    pub data_type: String,
    pub column_default: Option<String>,
    pub rust_type: TokenStream,
    pub rust_type_for_new: TokenStream,
    pub nullable_rust_type: TokenStream,
    pub is_primary_key: bool,
    pub udt_name: String,
}

fn make_fn_delete(table_name: &String, colums: &Vec<Column>) -> TokenStream {
    let (key_columns, _rest_columns): (Vec<&Column>, Vec<&Column>) =
        colums.iter().partition(|cloumn| cloumn.is_primary_key);

    let where_sql = key_columns
        .iter()
        .enumerate()
        .map(|(index, column)| format!("{} = ${}", &column.name, index + 1))
        .collect::<Vec<_>>()
        .join(", ");
    let statement = format!("DELETE FROM {} WHERE {}", table_name, where_sql);

    let params = key_columns
        .iter()
        .map(|column| format_ident!("{}", &column.name))
        .collect::<Vec<_>>();
    let params = quote! { &[#(&self.#params),*] };

    quote! {
        pub async fn delete(&self, client: &tokio_postgres::Client) -> anyhow::Result<()> {
            client.execute(#statement, #params).await?;
            Ok(())
        }
    }
}

fn make_fn_insert(struct_ident: &Ident, table_name: &String, colums: &Vec<Column>) -> TokenStream {
    let (_key_columns, rest_columns): (Vec<&Column>, Vec<&Column>) =
        colums.iter().partition(|cloumn| cloumn.is_primary_key);

    let target_columns: Vec<TokenStream> = rest_columns
        .iter()
        .map(|column| {
            let name = format_ident!("{}", &column.name);
            if !column.is_nullable && column.column_default.is_some() {
                quote! {
                    if self.#name.is_some() {
                        target_columns.push(stringify!(#name));
                    }
                }
            } else {
                quote! {
                    target_columns.push(stringify!(#name));
                }
            }
        })
        .collect();

    let count_bind: Vec<TokenStream> = rest_columns
        .iter()
        .map(|column| {
            let name = format_ident!("{}", &column.name);
            if !column.is_nullable && column.column_default.is_some() {
                quote! {
                    if self.#name.is_some() {
                        bind_count += 1;
                    }
                }
            } else {
                quote! {
                    bind_count += 1;
                }
            }
        })
        .collect();

    let params: Vec<TokenStream> = rest_columns
        .iter()
        .map(|column| {
            let name = format_ident!("{}", &column.name);
            if !column.is_nullable && column.column_default.is_some() {
                quote! {
                    if self.#name.is_some() {
                        params.push(&self.#name);
                    }
                }
            } else {
                quote! {
                    params.push(&self.#name);
                }
            }
        })
        .collect();

    quote! {
        pub async fn insert(&self, client: &tokio_postgres::Client) -> anyhow::Result<#struct_ident> {
            let mut target_columns: Vec<&str> = vec![];
            #(#target_columns)*
            let target_columns = target_columns.join(", ");

            let mut bind_count: i32 = 0;
            #(#count_bind)*
            let binds = (1..=bind_count).map(|i| format!("${}", i)).collect::<Vec<_>>().join(", ");

            let statement = format!(
                "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
                #table_name, target_columns, binds
            );

            let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
            #(#params)*

            let row = client.query_one(statement.as_str(), &params[..]).await?;
            Ok(row.into())
        }
    }
}

fn make_fn_update(table_name: &String, colums: &Vec<Column>) -> TokenStream {
    let (key_columns, rest_columns): (Vec<&Column>, Vec<&Column>) =
        colums.iter().partition(|cloumn| cloumn.is_primary_key);

    let set_sql = rest_columns
        .iter()
        .enumerate()
        .map(|(index, column)| format!("{} = ${}", &column.name, index + 1))
        .collect::<Vec<_>>()
        .join(", ");
    let where_sql = key_columns
        .iter()
        .enumerate()
        .map(|(index, column)| format!("{} = ${}", &column.name, index + rest_columns.len() + 1))
        .collect::<Vec<_>>()
        .join(", ");
    let statement = format!("UPDATE {} SET {} WHERE {}", table_name, set_sql, where_sql);

    let params = rest_columns
        .iter()
        .chain(key_columns.iter())
        .map(|column| format_ident!("{}", &column.name))
        .collect::<Vec<_>>();
    let params = quote! { &[#(&self.#params),*] };

    quote! {
        pub async fn update(&self, client: &tokio_postgres::Client) -> anyhow::Result<()> {
            client.execute(#statement, #params).await?;
            Ok(())
        }
    }
}

fn uniq_use(
    x: Vec<Vec<TokenStream>>,
    y: Vec<Vec<TokenStream>>,
    z: Vec<Vec<TokenStream>>,
) -> Vec<TokenStream> {
    let mut x: Vec<TokenStream> = x
        .into_iter()
        .chain(y.into_iter())
        .chain(z.into_iter())
        .flatten()
        .collect();
    x.sort_by_key(|x| x.to_string());
    x.dedup_by_key(|x| x.to_string());
    x
}
