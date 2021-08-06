use crate::error::Result;
use crate::generator::Column;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashMap;
use tokio_postgres::Client;

pub async fn definitions(
    columns: &Vec<Column>,
    client: &Client,
) -> Result<HashMap<String, TokenStream>> {
    let mut result = HashMap::new();
    for column in columns.iter() {
        if column.data_type != "USER-DEFINED" || column.udt_name == "geography" {
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
            .map(|x| {
                let rust_name = crate::utils::title_case(x);
                format_ident!("{}", rust_name)
            })
            .collect();
        let enum_name = &column.rust_type;
        let enum_name_pg = &column.udt_name;
        result.insert(
            enum_name.to_string(),
            quote! {
                #[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
                #[cfg_attr(
                    target_arch = "x86_64",
                    derive(FromSql, ToSql),
                    postgres(name = #enum_name_pg)
                )]
                pub enum #enum_name {
                    #(#[cfg_attr(
                        target_arch = "x86_64",
                        postgres(name = #enumlabels_pg)
                    )]
                    #enumlabels,)*
                }
            },
        );
    }
    Ok(result)
}
