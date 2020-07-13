use crate::Args;
use inflector::Inflector;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

pub struct HasMany {
    pub has_many_field: TokenStream,
    pub has_many_init: TokenStream,
    pub has_many_builder_field: TokenStream,
    pub has_many_builder_impl: TokenStream,
    pub has_many_filters_impl: TokenStream,
    pub has_many_join: TokenStream,
    pub has_many_preload: TokenStream,
}

pub fn make_has_many(
    args: &Args,
    _self_struct_name: &Ident,
    self_table_name: &String,
    self_builder_name: &Ident,
) -> HasMany {
    match args.get("has_many") {
        Some(field_name) => {
            let foreign_key = format!("{}_id", self_table_name.to_singular());
            let join = format!(
                " INNER JOIN {} ON {}.{} = {}.id",
                field_name.to_string(),
                field_name.to_string(),
                foreign_key,
                self_table_name
            );
            let struct_name = format_ident!("{}", field_name.to_string().to_class_case());
            let builder_field = format_ident!("{}_builder", field_name.to_string());
            let child_builder_name = format_ident!("{}Builder", &struct_name.to_string());
            HasMany {
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
                    result.append(
                        &mut self.#builder_field.as_ref()
                            .map_or(vec![],
                                    |x| x.filters.iter().collect::<Vec<&Filter>>())
                    );
                },
                has_many_join: quote! {
                    if self.#builder_field.is_some() {
                        result.push_str(#join);
                    }
                },
                has_many_preload: quote! {
                    if self.#builder_field.as_ref().map_or(false, |x| x.preload) {
                        let ids = xs.iter().map(|x| x.id).collect::<Vec<_>>();
                        let roles = Role::select().user_id().eq_any(ids).load(client).await?;
                        xs.iter_mut().for_each(|x| {
                            let mut ys = vec![];
                            for role in roles.iter() {
                                if x.id == role.user_id {
                                    ys.push(role.clone());
                                }
                            }
                            x.roles = Some(ys);
                        });
                    }
                },
            }
        }
        None => HasMany {
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
