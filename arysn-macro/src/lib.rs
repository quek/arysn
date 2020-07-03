extern crate proc_macro;

use anyhow::Result;
use log::debug;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use tokio::runtime::Runtime;
use tokio_postgres::{Client, NoTls};

#[proc_macro]
pub fn defar(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    env_logger::init();
    let args = syn::parse_macro_input!(input as Args);
    debug!("args {:?}", &args);
    let output: TokenStream = impl_defar(args).unwrap();
    proc_macro::TokenStream::from(output)
}

fn impl_defar(args: Args) -> Result<TokenStream> {
    let mut rt = Runtime::new()?;

    rt.block_on(async {
        let client = connect().await?;
        let sql = format!(
            r"
SELECT column_name, is_nullable, data_type
FROM
  information_schema.columns
WHERE
  table_name = '{}'
ORDER BY ordinal_position
",
            args.table_name
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

#[derive(Debug)]
struct Args {
    struct_name: Ident,
    table_name: Ident,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> std::result::Result<Self, syn::Error> {
        Ok(Self {
            struct_name: input.parse()?,
            table_name: input.parse()?,
        })
    }
}
