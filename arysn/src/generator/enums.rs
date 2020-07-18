use crate::generator::Column;
use anyhow::Result;
use inflector::Inflector;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use tokio_postgres::Client;

pub async fn definitions(columns: &Vec<Column>, client: &Client) -> Result<Vec<TokenStream>> {
    let mut result = vec![];
    for column in columns.iter() {
        if column.data_type != "USER-DEFINED" {
            continue;
        }
        let sql = r"
SELECT e.enumlabel
FROM pg_type t
   JOIN pg_enum e ON t.oid = e.enumtypid
   JOIN pg_catalog.pg_namespace n ON n.oid = t.typnamespace
WHERE t.typname = $1
ORDER BY e.enumsortorder
";
        let rows = client.query(sql, &[&column.udt_name]).await?;
        let enumlabels_pg: Vec<String> = rows.iter().map(|row| row.get(0)).collect();
        let enumlabels: Vec<Ident> = enumlabels_pg
            .iter()
            .map(|x| format_ident!("{}", x.to_class_case()))
            .collect();
        let enum_name = &column.rust_type;
        let enum_name_pg = &column.udt_name;
        result.push(quote! {
            #[derive(Debug, Clone, ToSql, FromSql)]
            #[postgres(name = #enum_name_pg)]
            pub enum #enum_name {
                #(#[postgres(name = #enumlabels_pg)]
                #enumlabels,)*
            }
        });
    }
    Ok(result)
}