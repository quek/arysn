extern crate proc_macro;

use anyhow::Result;
use log::debug;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{braced, Token};
use tokio::runtime::Runtime;
use tokio_postgres::{Client, NoTls};

#[proc_macro]
pub fn defar(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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
    let output: TokenStream = impl_defar(args).unwrap();
    proc_macro::TokenStream::from(output)
}

async fn columns(table_name: &String, client: &Client) -> Result<Vec<Column>> {
    let primary_key: Vec<String> = primary_key(table_name, client).await?;
    let sql = format!(
        r"
SELECT column_name, is_nullable, data_type
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
            let rust_type = rust_type(&data_type, is_nullable);
            let is_primary_key = primary_key.iter().any(|x| x == &name);
            Column {
                name,
                is_nullable,
                data_type,
                rust_type,
                is_primary_key,
            }
        })
        .collect();
    Ok(result)
}

fn impl_defar(args: Args) -> Result<TokenStream> {
    let mut rt = Runtime::new()?;

    rt.block_on(async {
        let client = connect().await?;

        let table_name = args
            .fields
            .iter()
            .find(|field| {
                field
                    .ident
                    .as_ref()
                    .map(|x| x.to_string().as_str() == "table_name")
                    .unwrap_or(false)
            })
            .map(|field| {
                let ty = &field.ty;
                let x = quote! { #ty };
                x.to_string()
            })
            .expect("no table_name field!");

        let columns: Vec<Column> = columns(&table_name, &client).await?;

        let mut column_names = Vec::<Ident>::new();
        let mut rust_types = Vec::<TokenStream>::new();
        for column in columns.iter() {
            column_names.push(Ident::new(&column.name, Span::call_site()));
            rust_types.push(column.rust_type.clone());
        }

        let name = &args.struct_name;
        let builder_name = Ident::new(&format!("{}Builder", &args.struct_name), Span::call_site());
        let builder_name_columns: Vec<Ident> = columns
            .iter()
            .map(|column| {
                Ident::new(
                    &format!("{}_{}", &builder_name, &column.name),
                    Span::call_site(),
                )
            })
            .collect();

        let output = quote! {
            #[derive(Debug)]
            struct #name {
                #(pub #column_names: #rust_types),*
            }

            impl From<tokio_postgres::row::Row> for #name {
                fn from(row: tokio_postgres::row::Row) -> Self {
                    Self {
                        id: row.get(0),
                        name: row.get(1),
                        title: row.get(2),
                    }
                }
            }

            impl #name {
                pub fn filter<T: std::fmt::Display>(value: T) -> Builder {
                    Builder::default().from(#table_name.to_string()).filter(value)
                }

                pub fn select() -> #builder_name {
                    #builder_name::default()
                }
            }

            #[derive(Clone, Debug, Default)]
            struct #builder_name {
                pub filters: Vec<String>
            }

            impl #builder_name {
                #(pub fn #column_names(&self) -> #builder_name_columns {
                    #builder_name_columns {
                        builder: self.clone()
                    }
                })*
            }

            impl BuilderTrait for #builder_name {
            }

            #(
                struct #builder_name_columns {
                    pub builder: #builder_name,
                }
                impl #builder_name_columns {
                    pub fn eq(&self, value: #rust_types) -> #builder_name {
                        let mut filters = self.builder.filters.clone();
                        filters.push(format!("{}={:?}", stringify!(#column_names), value));
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

fn rust_type(data_type: &str, is_nullable: bool) -> TokenStream {
    let rust_type = match data_type {
        "bigint" => quote!(i64),
        "character varying" => quote!(String),
        _ => panic!("unknown sql type: {}", data_type),
    };
    if is_nullable {
        quote!(Option<#rust_type>)
    } else {
        rust_type
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
    pub rust_type: TokenStream,
    pub is_primary_key: bool,
}
