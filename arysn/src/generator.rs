use anyhow::Result;
use belongs_to::{make_belongs_to, BelongsTo};
use config::Config;
use has_many::{make_has_many, HasMany};
use log::debug;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::io::Write;
use std::process::Command;
use tokio::runtime::Runtime;
use tokio_postgres::{Client, NoTls};

mod belongs_to;
pub mod config;
mod has_many;

pub fn define_ar(config: &Config) -> Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();

    let output: TokenStream = define_ar_impl(config).unwrap();
    {
        let mut writer = std::io::BufWriter::new(std::fs::File::create(config.path)?);
        writeln!(writer, "{}", &output.to_string())?;
    }
    Command::new("rustfmt")
        .arg("--edition")
        .arg("2018")
        .arg(config.path)
        .output()?;
    Ok(())
}

async fn columns(table_name: &String, client: &Client) -> Result<Vec<Column>> {
    let primary_key: Vec<String> = primary_key(table_name, client).await?;
    let sql = format!(
        r"
SELECT column_name, is_nullable, data_type, column_default
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
            let (rust_type, mut nullable_rust_type, value_type) =
                compute_type(&data_type, is_nullable);
            let rust_type_for_new = if column_default.is_some() && !is_nullable {
                quote! { Option<#rust_type> }
            } else {
                nullable_rust_type.clone()
            };
            let is_primary_key = primary_key.iter().any(|x| x == &name);
            if !is_primary_key && column_default.is_some() {
                nullable_rust_type = quote!(Option<#rust_type>);
            }
            Column {
                name,
                is_nullable,
                data_type,
                column_default,
                rust_type,
                rust_type_for_new,
                nullable_rust_type,
                value_type,
                is_primary_key,
            }
        })
        .collect();
    Ok(result)
}

fn define_ar_impl(config: &Config) -> Result<TokenStream> {
    let mut rt = Runtime::new()?;

    rt.block_on(async {
        let client = connect().await?;

        let table_name: String = config.table_name.to_string();
        let columns: Vec<Column> = columns(&table_name, &client).await?;

        let mut column_names = Vec::<Ident>::new();
        let mut rust_types = Vec::<TokenStream>::new();
        let mut rust_types_for_new = Vec::<TokenStream>::new();
        let mut nullable_rust_types = Vec::<TokenStream>::new();
        let mut value_types = Vec::<TokenStream>::new();
        for column in columns.iter() {
            column_names.push(format_ident!("{}", &column.name));
            rust_types.push(column.rust_type.clone());
            rust_types_for_new.push(column.rust_type_for_new.clone());
            nullable_rust_types.push(column.nullable_rust_type.clone());
            value_types.push(column.value_type.clone());
        }
        let column_index = 0..columns.len();

        let struct_name: &Ident = &config.struct_name;
        let new_struct_name: Ident = format_ident!("{}New", struct_name);
        let builder_name: Ident = format_ident!("{}Builder", struct_name);
        let builder_name_columns: Vec<Ident> = columns
            .iter()
            .map(|column| format_ident!("{}_{}", &builder_name, &column.name))
            .collect();

        let fn_delete: TokenStream = make_fn_delete(&table_name, &columns);
        let fn_insert: TokenStream = make_fn_insert(struct_name, &table_name, &columns);
        let fn_update: TokenStream = make_fn_update(&table_name, &columns);

        let HasMany {
            has_many_use,
            has_many_field,
            has_many_init,
            has_many_builder_field,
            has_many_builder_impl,
            has_many_filters_impl,
            has_many_join,
            has_many_preload,
        } = make_has_many(config, &builder_name);

        let BelongsTo {
            belongs_to_use,
            belongs_to_field,
            belongs_to_init,
            belongs_to_builder_field,
            belongs_to_builder_impl,
            belongs_to_filters_impl,
            belongs_to_join,
            belongs_to_preload,
        } = make_belongs_to(config, &builder_name);

        let output = quote! {
            use arysn::prelude::*;
            use async_recursion::async_recursion;
            #(#has_many_use)*
            #(#belongs_to_use)*

            #[derive(Clone, Debug)]
            pub struct #struct_name {
                #(pub #column_names: #nullable_rust_types,)*
                #(#has_many_field)*
                #(#belongs_to_field)*
            }

            #[derive(Clone, Debug)]
            pub struct #new_struct_name {
                #(pub #column_names: #rust_types_for_new,)*
            }

            impl #struct_name {
                pub fn select() -> #builder_name {
                    #builder_name {
                        from: #table_name.to_string(),
                        ..#builder_name::default()
                    }
                }
                #fn_delete
                #fn_update
            }

            impl #new_struct_name {
                #fn_insert
            }

            impl From<tokio_postgres::row::Row> for #struct_name {
                fn from(row: tokio_postgres::row::Row) -> Self {
                    Self {
                        #(
                            #column_names: row.get(#column_index),
                        )*
                        #(#has_many_init)*
                        #(#belongs_to_init)*
                    }
                }
            }

            #[derive(Clone, Debug, Default)]
            pub struct #builder_name {
                pub from: String,
                pub filters: Vec<Filter>,
                pub preload: bool,
                pub order: String,
                #(#has_many_builder_field)*
                #(#belongs_to_builder_field)*
            }

            impl #builder_name {
                #(pub fn #column_names(&self) -> #builder_name_columns {
                    #builder_name_columns {
                        builder: self.clone()
                    }
                })*
                #(#has_many_builder_impl)*
                #(#belongs_to_builder_impl)*

                pub fn order<T: AsRef<str>>(&self, value: T) -> Self {
                    Self {
                        order: value.as_ref().to_string(),
                        ..self.clone()
                    }
                }

                pub fn preload(&self) -> Self {
                    Self {
                        preload: true,
                        ..self.clone()
                    }
                }

                pub async fn first(&self, client: &tokio_postgres::Client) ->
                    anyhow::Result<#struct_name> {
                    let params = self.select_params();
                    let row = client
                        .query_one(self.select_sql().as_str(), &params[..])
                        .await?;
                    let x: #struct_name = #struct_name::from(row);
                    Ok(x)
                }

                #[async_recursion]
                pub async fn load(&self, client: &tokio_postgres::Client) ->
                    anyhow::Result<Vec<#struct_name>> {
                    let params = self.select_params();
                    let rows = client
                        .query(self.select_sql().as_str(), &params[..])
                        .await?;
                    let mut result: Vec<#struct_name> = rows.into_iter()
                            .map(|row| #struct_name::from(row)).collect();
                    #(#has_many_preload)*
                    #(#belongs_to_preload)*
                    Ok(result)
                }
            }

            impl BuilderTrait for #builder_name {
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
                    #(#belongs_to_join)*
                }

                fn filters(&self) -> Vec<&Filter> {
                    let mut result: Vec<&Filter> = self.filters.iter().collect();
                    #(#has_many_filters_impl)*
                    #(#belongs_to_filters_impl)*
                    result
                }

                fn order_part(&self) -> String {
                    self.order.clone()
                }
            }

            #(
                #[allow(non_camel_case_types)]
                pub struct #builder_name_columns {
                    pub builder: #builder_name,
                }
                impl #builder_name_columns {
                    pub fn eq(&self, value: #rust_types) -> #builder_name {
                        let mut filters = self.builder.filters.clone();
                        filters.push(Filter {
                            table: #table_name.to_string(),
                            name: stringify!(#column_names).to_string(),
                            value: value.into(),
                            operator: "=".to_string()
                        });
                        #builder_name {
                            filters,
                            ..self.builder.clone()
                        }
                    }

                    pub fn eq_any(&self, value: Vec<#rust_types>) -> #builder_name {
                        let mut filters = self.builder.filters.clone();
                        filters.push(Filter {
                            table: #table_name.to_string(),
                            name: stringify!(#column_names).to_string(),
                            value: value.into(),
                            operator: "in".to_string(),
                        });
                        #builder_name {
                            filters,
                            ..self.builder.clone()
                        }
                    }
                }
            )*
        };
        debug!("output: {}", &output);
        Ok(output.into())
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

fn compute_type(data_type: &str, is_nullable: bool) -> (TokenStream, TokenStream, TokenStream) {
    let (rust_type, value_type) = match data_type {
        "boolean" => (quote!(bool), quote!(Bool)),
        "integer" => (quote!(i32), quote!(I32)),
        "bigint" => (quote!(i64), quote!(I64)),
        "character varying" => (quote!(String), quote!(String)),
        "timestamp with time zone" => (quote!(chrono::DateTime<chrono::Local>), quote!(DateTime)),
        _ => panic!("unknown sql type: {}", data_type),
    };
    if is_nullable {
        (rust_type.clone(), quote!(Option<#rust_type>), value_type)
    } else {
        (rust_type.clone(), rust_type, value_type)
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

struct Column {
    pub name: String,
    pub is_nullable: bool,
    pub data_type: String,
    pub column_default: Option<String>,
    pub rust_type: TokenStream,
    pub rust_type_for_new: TokenStream,
    pub nullable_rust_type: TokenStream,
    pub value_type: TokenStream,
    pub is_primary_key: bool,
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

fn make_fn_insert(struct_name: &Ident, table_name: &String, colums: &Vec<Column>) -> TokenStream {
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
        pub async fn insert(&self, client: &tokio_postgres::Client) -> anyhow::Result<#struct_name> {
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
