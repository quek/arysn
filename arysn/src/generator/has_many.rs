use crate::generator::config::Config;
use inflector::Inflector;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

pub struct HasMany {
    pub has_many_use: TokenStream,
    pub has_many_field: TokenStream,
    pub has_many_init: TokenStream,
    pub has_many_builder_field: TokenStream,
    pub has_many_builder_impl: TokenStream,
    pub has_many_filters_impl: TokenStream,
    pub has_many_join: TokenStream,
    pub has_many_preload: TokenStream,
}

pub fn make_has_many(config: &Config, self_builder_name: &Ident) -> HasMany {
    match &config.has_many {
        Some(has_many) => {
            let module_name = format_ident!(
                "{}",
                has_many
                    .struct_name
                    .to_string()
                    .to_table_case()
                    .to_singular()
            );
            let foreign_key = format_ident!("{}_id", config.table_name.to_singular());
            let field_name = &has_many.field;
            let join = format!(
                "INNER JOIN {} ON {}.{} = {}.id",
                field_name.to_string(),
                field_name.to_string(),
                foreign_key.to_string(),
                config.table_name,
            );
            let struct_name = &has_many.struct_name;
            let builder_field = format_ident!("{}_builder", field_name.to_string());
            let child_builder_name = format_ident!("{}Builder", &struct_name.to_string());
            HasMany {
                has_many_use: quote! {
                    use super::#module_name::{#struct_name, #child_builder_name};
                },
                has_many_field: quote! { pub #field_name: Option<Vec<#struct_name>>, },
                has_many_init: quote! { #field_name: None, },
                has_many_builder_field: quote! { pub #builder_field: Option<Box<#child_builder_name>>, },
                has_many_builder_impl: quote! {
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
                has_many_filters_impl: quote! {
                    if let Some(builder) = &self.#builder_field {
                        result.append(&mut builder.filters());
                    }
                },
                has_many_join: quote! {
                    if let Some(builder) = &self.#builder_field {
                        join_parts.push(#join.to_string());
                        builder.join(join_parts);
                    }
                },
                has_many_preload: quote! {
                    if let Some(builder) = &self.#builder_field {
                        if builder.preload {
                            let ids = result.iter().map(|x| x.id).collect::<Vec<_>>();
                            let children_builder = #struct_name::select().#foreign_key().eq_any(ids);
                            let children_builder = #child_builder_name {
                                from: children_builder.from,
                                filters: children_builder.filters,
                                ..(**builder).clone()
                            };
                            let children = children_builder.load(client).await?;
                            result.iter_mut().for_each(|x| {
                                let mut ys = vec![];
                                for child in children.iter() {
                                    if x.id == child.#foreign_key {
                                        ys.push(child.clone());
                                    }
                                }
                                x.#field_name = Some(ys);
                            });
                        }
                    }
                },
            }
        }
        None => HasMany {
            has_many_use: quote!(),
            has_many_field: quote!(),
            has_many_init: quote!(),
            has_many_builder_field: quote!(),
            has_many_builder_impl: quote!(),
            has_many_filters_impl: quote!(),
            has_many_join: quote!(),
            has_many_preload: quote!(),
        },
    }
}
