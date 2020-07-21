use crate::generator::Column;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

pub fn order_part(
    struct_ident: &Ident,
    builder_ident: &Ident,
    columns: &Vec<Column>,
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
                    field: self.field,
                    asc_or_desc: "ASC",
                });
                builder
            }

            pub fn desc(&self) -> #builder_ident {
                let mut builder = self.order_builder.builder.clone();
                builder.orders.push(OrderItem {
                    field: self.field,
                    asc_or_desc: "DESC",
                });
                builder
            }
        }
    }
}
