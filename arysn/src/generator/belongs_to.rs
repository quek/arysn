use crate::generator::config::Config;
use inflector::Inflector;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

pub struct BelongsTo {
    pub belongs_to_use: TokenStream,
    pub belongs_to_field: TokenStream,
    pub belongs_to_init: TokenStream,
    pub belongs_to_builder_field: TokenStream,
    pub belongs_to_builder_impl: TokenStream,
    pub belongs_to_filters_impl: TokenStream,
    pub belongs_to_join: TokenStream,
}

pub fn make_belongs_to(config: &Config, self_builder_name: &Ident) -> BelongsTo {
    match &config.belongs_to {
        Some(belongs_to) => {
            let module_name = format_ident!(
                "{}",
                belongs_to
                    .struct_name
                    .to_string()
                    .to_table_case()
                    .to_singular()
            );
            let field_name = &belongs_to.field;
            let foreign_key = format!("{}_id", &field_name);
            let join = format!(
                " INNER JOIN {} ON {}.id = {}.{}",
                field_name.to_string().to_table_case(),
                field_name.to_string().to_table_case(),
                config.table_name,
                foreign_key
            );
            let struct_name = &belongs_to.struct_name;
            let builder_field = format_ident!("{}_bulider", &field_name);
            let child_builder_name = format_ident!("{}Builder", &struct_name.to_string());
            BelongsTo {
                belongs_to_use: quote! {
                    use super::#module_name::{#struct_name, #child_builder_name};
                },
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
            belongs_to_use: quote!(),
            belongs_to_field: quote!(),
            belongs_to_init: quote!(),
            belongs_to_builder_field: quote!(),
            belongs_to_builder_impl: quote!(),
            belongs_to_filters_impl: quote!(),
            belongs_to_join: quote!(),
        },
    }
}
