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
        let mut column_names = Vec::<Ident>::new();
        let mut rust_types = Vec::<TokenStream>::new();
        for row in rows {
            let column_name: &str = row.get(0);
            column_names.push(Ident::new(column_name, Span::call_site()));
            let is_nullable: &str = row.get(1);
            let is_nullable: bool = is_nullable == "YES";
            let data_type: &str = row.get(2);
            let rust_type = rust_type(data_type, is_nullable);
            rust_types.push(rust_type);
        }

        let name = &args.struct_name;
        let output = quote! {
            #[derive(Debug)]
            struct #name {
                #(pub #column_names: #rust_types),*
            }

            impl #name {
                pub fn filter<T: std::fmt::Display>(value: T) -> Builder {
                    Builder::default().from(#table_name.to_string()).filter(value)
                }
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
        };
        debug!("output: {}", &output);
        Ok(output.into())
    })
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
