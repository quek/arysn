extern crate proc_macro;

use anyhow::Result;
use inflector::Inflector;
use log::debug;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{braced, Token};
use tokio::runtime::Runtime;
use tokio_postgres::{Client, NoTls};

#[proc_macro]
pub fn define_ar(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = env_logger::builder().is_test(true).try_init();

    // TODO input.to_string() で "User { table_name : users }" になるからそれを JSON 処理しちゃう？
    debug!("input {:?}", &input);
    input.clone().into_iter().for_each(|x| println!("{:?}", x));
    debug!("input {}", input.to_string());

    {
        debug!("-----------------------------------------------------------------------------");
        let mut iter = input.clone().into_iter();
        let struct_name = iter.clone().nth(0).unwrap();
        debug!("struct name {:?}", &struct_name);
        if let proc_macro::TokenTree::Group(xs) = iter.nth(1).unwrap() {
            for x in xs.stream().into_iter() {
                debug!("{:?}", &x);
            }
        }
    }

    let args = syn::parse_macro_input!(input as Args);
    let output: TokenStream = define_ar_impl(args).unwrap();
    proc_macro::TokenStream::from(output)
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
            let is_primary_key = primary_key.iter().any(|x| x == &name);
            if is_primary_key && column_default.is_some() {
                nullable_rust_type = quote!(Option<#rust_type>);
            }
            Column {
                name,
                is_nullable,
                data_type,
                column_default,
                rust_type,
                nullable_rust_type,
                value_type,
                is_primary_key,
            }
        })
        .collect();
    Ok(result)
}

fn define_ar_impl(args: Args) -> Result<TokenStream> {
    let mut rt = Runtime::new()?;

    rt.block_on(async {
        let client = connect().await?;

        let table_name: String = args
            .get("table_name")
            .expect("table_name field is required!")
            .to_string();
        let columns: Vec<Column> = columns(&table_name, &client).await?;

        let mut column_names = Vec::<Ident>::new();
        let mut rust_types = Vec::<TokenStream>::new();
        let mut nullable_rust_types = Vec::<TokenStream>::new();
        let mut value_types = Vec::<TokenStream>::new();
        for column in columns.iter() {
            column_names.push(format_ident!("{}", &column.name));
            rust_types.push(column.rust_type.clone());
            nullable_rust_types.push(column.nullable_rust_type.clone());
            value_types.push(column.value_type.clone());
        }
        let column_index = 0..columns.len();

        let struct_name: &Ident = &args.struct_name;
        let builder_name: Ident = format_ident!("{}Builder", struct_name);
        let builder_name_columns: Vec<Ident> = columns
            .iter()
            .map(|column| format_ident!("{}_{}", &builder_name, &column.name))
            .collect();

        let fn_delete: TokenStream = make_fn_delete(&table_name, &columns);
        let fn_insert: TokenStream = make_fn_insert(&table_name, &columns);
        let fn_update: TokenStream = make_fn_update(&table_name, &columns);

        let HasMany {
            has_many_field,
            has_many_init,
            has_many_builder_field,
            has_many_builder_impl,
            has_many_filters_impl,
            has_many_join,
        } = make_has_many(&args, &builder_name);

        let output = quote! {
            #[derive(Clone, Debug)]
            pub struct #struct_name {
                #(pub #column_names: #nullable_rust_types,)*
                #has_many_field
            }

            impl #struct_name {
                pub fn select() -> #builder_name {
                    #builder_name {
                        from: #table_name.to_string(),
                        ..#builder_name::default()
                    }
                }
                #fn_delete
                #fn_insert
                #fn_update
            }

            impl From<tokio_postgres::row::Row> for #struct_name {
                fn from(row: tokio_postgres::row::Row) -> Self {
                    Self {
                        #(
                            #column_names: row.get(#column_index),
                        )*
                        #has_many_init
                    }
                }
            }

            #[derive(Clone, Debug, Default)]
            pub struct #builder_name {
                pub from: String,
                pub filters: Vec<Filter>,
                #has_many_builder_field
            }

            impl #builder_name {
                #(pub fn #column_names(&self) -> #builder_name_columns {
                    #builder_name_columns {
                        builder: self.clone()
                    }
                })*
                #has_many_builder_impl
                pub async fn first(&self, client: &tokio_postgres::Client) ->
                    anyhow::Result<#struct_name> {
                    let params = self.select_params();
                    let row = client
                        .query_one(self.select_sql().as_str(), &params[..])
                        .await?;
                    let x: #struct_name = #struct_name::from(row);
                    Ok(x)
                }
                pub async fn load(&self, client: &tokio_postgres::Client) ->
                    anyhow::Result<Vec<#struct_name>> {
                    let params = self.select_params();
                    let rows = client
                        .query(self.select_sql().as_str(), &params[..])
                        .await?;
                    let xs: Vec<#struct_name> = rows.into_iter()
                        .map(|row| #struct_name::from(row)).collect();
                    Ok(xs)
                }
            }

            impl BuilderTrait for #builder_name {
                fn select(&self) -> String {
                    #table_name.to_string()
                }

                fn from(&self) -> String {
                    let mut result = self.from.clone();
                    #has_many_join
                    result
                }

                fn filters(&self) -> Vec<&Filter> {
                    let mut result: Vec<&Filter> = self.filters.iter().collect();
                    #has_many_filters_impl
                    result
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
                            value: Value::#value_types(value),
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
    client.execute("SET log_statement = 'all'", &[]).await?;
    client.execute("SET TIME ZONE 'Japan'", &[]).await?;
    Ok(client)
}

struct Args {
    struct_name: Ident,
    _brace_token: syn::token::Brace,
    fields: syn::punctuated::Punctuated<syn::Field, Token![,]>,
}

impl Args {
    pub fn get(&self, key: &str) -> Option<TokenStream> {
        self.fields
            .iter()
            .find(|field| {
                field
                    .ident
                    .as_ref()
                    .map(|x| x.to_string().as_str() == key)
                    .unwrap_or(false)
            })
            .map(|field| {
                let ty = &field.ty;
                let x = quote! { #ty };
                x
            })
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> std::result::Result<Self, syn::Error> {
        let content;
        Ok(Self {
            struct_name: input.parse()?,
            _brace_token: braced!(content in input),
            fields: content.parse_terminated(syn::Field::parse_named)?,
        })
    }
}

struct Column {
    pub name: String,
    pub is_nullable: bool,
    pub data_type: String,
    pub column_default: Option<String>,
    pub rust_type: TokenStream,
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

fn make_fn_insert(table_name: &String, colums: &Vec<Column>) -> TokenStream {
    let (_key_columns, rest_columns): (Vec<&Column>, Vec<&Column>) =
        colums.iter().partition(|cloumn| cloumn.is_primary_key);

    let target_columns = rest_columns
        .iter()
        .map(|column| column.name.clone())
        .collect::<Vec<_>>()
        .join(", ");

    let binds = rest_columns
        .iter()
        .enumerate()
        .map(|(index, _column)| format!("${}", index + 1))
        .collect::<Vec<_>>()
        .join(", ");

    let statement = format!(
        "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
        table_name, target_columns, binds
    );

    let params = rest_columns
        .iter()
        .map(|column| format_ident!("{}", &column.name))
        .collect::<Vec<_>>();
    let params = quote! { &[#(&self.#params),*] };

    quote! {
        pub async fn insert(&self, client: &tokio_postgres::Client) -> anyhow::Result<Self> {
            let row = client.query_one(#statement, #params).await?;
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

// fn make_belogns_to(args: &Args) -> (TokenStream, TokenStream, TokenStream, TokenStream) {
//     match args.get("bolongs_to") {
//         Some(field_name) => {
//             let struct_name = format_ident!("{}", field_name.to_string().to_class_case());
//             let builder_field = format_ident!("{}_bulider", field_name.to_string());
//             let builder_type = format_ident!("{}Builder", &struct_name.to_string());
//             (
//                 quote! { pub #field_name: Option<#struct_name>, },
//                 quote! { #field_name: None, },
//                 quote! { pub #builder_field: #builder_type, },
//                 quote! {
//                     pub fn #field_name(&self) -> #builder_type {
//                         #builder_type {
//                             parent: Some(self.clone()),
//                             ..#builder_type::default()
//                         }
//                     }
//                 },
//             )
//         }
//         None => (quote!(), quote!(), quote!(), quote!()),
//     }
// }

struct HasMany {
    has_many_field: TokenStream,
    has_many_init: TokenStream,
    has_many_builder_field: TokenStream,
    has_many_builder_impl: TokenStream,
    has_many_filters_impl: TokenStream,
    has_many_join: TokenStream,
}

fn make_has_many(args: &Args, self_builder_name: &Ident) -> HasMany {
    match args.get("has_many") {
        Some(field_name) => {
            let struct_name = format_ident!("{}", field_name.to_string().to_class_case());
            let builder_field = format_ident!("{}_builder", field_name.to_string());
            let child_builder_name = format_ident!("{}Builder", &struct_name.to_string());
            HasMany {
                has_many_field: quote! { pub #field_name: Option<Vec<#struct_name>>, },
                has_many_init: quote! { #field_name: None, },
                has_many_builder_field: quote! { pub #builder_field: Option<#child_builder_name>, },
                has_many_builder_impl: quote! {
                    pub fn #field_name<F>(&self, f: F) -> #self_builder_name
                    where F: FnOnce(&#child_builder_name) -> #child_builder_name {
                        UserBuilder {
                            #builder_field: Some(
                                f(self.#builder_field.as_ref().unwrap_or(&Default::default()))
                            ),
                            ..self.clone()
                        }
                    }
                },
                has_many_filters_impl: quote! {
                    result.append(
                        &mut self.#builder_field.as_ref()
                            .map_or(vec![],
                                    |x| x.filters.iter().collect::<Vec<&Filter>>())
                    );
                },
                has_many_join: quote! {
                    if self.#builder_field.is_some() {
                        result.push_str(" INNER JOIN roles ON roles.user_id = users.id");
                    }
                },
            }
        }
        None => HasMany {
            has_many_field: quote!(),
            has_many_init: quote!(),
            has_many_builder_field: quote!(),
            has_many_builder_impl: quote!(),
            has_many_filters_impl: quote!(),
            has_many_join: quote!(),
        },
    }
}
