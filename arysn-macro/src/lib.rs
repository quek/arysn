extern crate proc_macro;
use anyhow::Result;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use tokio::runtime::Runtime;
use tokio_postgres::{Client, NoTls};

#[proc_macro]
pub fn defar(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as Args);
    println!("args {:?}", &args);
    impl_defar(args).unwrap()
}

fn impl_defar(args: Args) -> Result<TokenStream> {
    let mut rt = Runtime::new()?;

    rt.block_on(async {
        let client = connect().await?;
        let sql = r"
SELECT column_name, is_nullable, data_type
FROM
  information_schema.columns
WHERE
  table_name = 'users'
ORDER BY ordinal_position
";
        let rows = client.query(sql, &[]).await?;
        let mut column_names = Vec::<Ident>::new();
        let mut is_nullables = Vec::<String>::new();
        let mut data_types = Vec::<String>::new();
        for row in rows {
            let s: &str = row.get(0);
            column_names.push(Ident::new(s, Span::call_site()));
            let s: &str = row.get(1);
            is_nullables.push(s.to_string());
            let s: &str = row.get(2);
            data_types.push(s.to_string());
        }

        let name = &args.name;
        let gen = quote! {
            #[derive(Debug)]
            struct #name {
                #(pub #column_names: String),*
            }
        };
        Ok(gen.into())
    })
}

async fn connect() -> Result<Client> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set!");
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    client.execute("SET log_statement = 'all'", &[]).await?;
    client.execute("SET TIME ZONE 'Japan'", &[]).await?;
    Ok(client)
}

#[derive(Debug)]
struct Args {
    name: Ident,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> std::result::Result<Self, syn::Error> {
        Ok(Self {
            name: input.parse()?,
        })
    }
}
