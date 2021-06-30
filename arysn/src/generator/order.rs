use crate::generator::Column;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

pub fn order_part(
    struct_ident: &Ident,
    builder_ident: &Ident,
    columns: &Vec<Column>,
    table_name: &String,
) -> TokenStream {
    let order_builder_ident = format_ident!("{}OrderBuilder", struct_ident);
    let asc_or_desc_ident = format_ident!("{}OrderAscOrDesc", struct_ident);
    let mut field_ident: Vec<Ident> = vec![];
    let mut field_name: Vec<String> = vec![];
    for column in columns.iter() {
        field_ident.push(format_ident!("{}", &column.name));
        field_name.push(column.name.clone());
    }

    quote! {
        impl #builder_ident {
            pub fn order(&self) -> #order_builder_ident {
                #order_builder_ident {
                    builder: self.clone(),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct #order_builder_ident {
            pub builder: #builder_ident,
        }

        impl #order_builder_ident {
            #(pub fn #field_ident(&self) -> #asc_or_desc_ident {
                #asc_or_desc_ident {
                    field: #field_name,
                    order_builder: self.clone(),
                }
            })*

            pub fn by_string_literal_asc(&self, field: &'static str) -> #builder_ident {
                let mut builder = self.builder.clone();
                builder.orders.push(OrderItem {
                    table: "".to_string(),
                    field,
                    asc_or_desc: "ASC",
                });
                builder
            }

            pub fn by_string_literal_desc(&self, field: &'static str) -> #builder_ident {
                let mut builder = self.builder.clone();
                builder.orders.push(OrderItem {
                    table: "".to_string(),
                    field,
                    asc_or_desc: "DESC",
                });
                builder
            }
        }

        #[derive(Clone, Debug)]
        pub struct #asc_or_desc_ident {
            pub field: &'static str,
            pub order_builder: #order_builder_ident,
        }

        impl #asc_or_desc_ident {
            pub fn asc(&self) -> #builder_ident {
                let mut builder = self.order_builder.builder.clone();
                builder.orders.push(OrderItem {
                    table: self.order_builder.builder.table_name_as.as_ref()
                        .unwrap_or(&#table_name.to_string()).to_string(),
                    field: self.field,
                    asc_or_desc: "ASC",
                });
                builder
            }

            pub fn desc(&self) -> #builder_ident {
                let mut builder = self.order_builder.builder.clone();
                builder.orders.push(OrderItem {
                    table: self.order_builder.builder.table_name_as.as_ref()
                        .unwrap_or(&#table_name.to_string()).to_string(),
                    field: self.field,
                    asc_or_desc: "DESC",
                });
                builder
            }
        }
    }
}
