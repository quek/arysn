use crate::Args;
use inflector::Inflector;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

pub struct BelongsTo {
    pub belongs_to_field: TokenStream,
    pub belongs_to_init: TokenStream,
    pub belongs_to_builder_field: TokenStream,
    pub belongs_to_builder_impl: TokenStream,
    pub belongs_to_filters_impl: TokenStream,
    pub belongs_to_join: TokenStream,
}

pub fn make_belongs_to(
    args: &Args,
    _self_struct_name: &Ident,
    self_table_name: &String,
    self_builder_name: &Ident,
) -> BelongsTo {
    match args.get("belongs_to") {
        Some(field_name) => {
            let foreign_key = format!("{}_id", field_name.to_string());
            let join = format!(
                " INNER JOIN {} ON {}.id = {}.{}",
                field_name.to_string().to_table_case(),
                field_name.to_string().to_table_case(),
                self_table_name,
                foreign_key
            );
            let struct_name = format_ident!("{}", field_name.to_string().to_class_case());
            let builder_field = format_ident!("{}_bulider", field_name.to_string());
            let child_builder_name = format_ident!("{}Builder", &struct_name.to_string());
            BelongsTo {
                belongs_to_field: quote! { pub #field_name: Option<#struct_name>, },
                belongs_to_init: quote! { #field_name: None, },
                belongs_to_builder_field: quote! { pub #builder_field: Option<Box<#child_builder_name>>, },
                belongs_to_builder_impl: quote! {
                    pub fn #field_name<F>(&self, f: F) -> #self_builder_name
                    where F: FnOnce(&#child_builder_name) -> #child_builder_name {
                        #self_builder_name {
                            #builder_field: Some(
                                Box::new(f(self.#builder_field.as_ref().unwrap_or(&Default::default())))
                            ),
                            ..self.clone()
                        }
                    }
                },
                belongs_to_filters_impl: quote! {
                    result.append(
                        &mut self.#builder_field.as_ref()
                            .map_or(vec![],
                                    |x| x.filters.iter().collect::<Vec<&Filter>>())
                    );
                },
                belongs_to_join: quote! {
                    if self.#builder_field.is_some() {
                        result.push_str(#join);
                    }
                },
            }
        }
        None => BelongsTo {
            belongs_to_field: quote!(),
            belongs_to_init: quote!(),
            belongs_to_builder_field: quote!(),
            belongs_to_builder_impl: quote!(),
            belongs_to_filters_impl: quote!(),
            belongs_to_join: quote!(),
        },
    }
}
