use crate::generator::config::Config;
use inflector::Inflector;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

#[derive(Default)]
pub struct BelongsTo {
    pub belongs_to_use: Vec<TokenStream>,
    pub belongs_to_field: Vec<TokenStream>,
    pub belongs_to_init: Vec<TokenStream>,
    pub belongs_to_builder_field: Vec<TokenStream>,
    pub belongs_to_builder_impl: Vec<TokenStream>,
    pub belongs_to_filters_impl: Vec<TokenStream>,
    pub belongs_to_join: Vec<TokenStream>,
    pub belongs_to_preload: Vec<TokenStream>,
}

pub fn make_belongs_to(config: &Config, self_builder_name: &Ident) -> BelongsTo {
    let mut result: BelongsTo = BelongsTo::default();
    for belongs_to in config.belongs_to.iter() {
        let module_name = format_ident!(
            "{}",
            belongs_to
                .struct_name
                .to_string()
                .to_table_case()
                .to_singular()
        );
        let field_name = &belongs_to.field;
        let foreign_key = format_ident!("{}_id", &field_name);
        let join = format!(
            "INNER JOIN {} ON {}.id = {}.{}",
            field_name.to_string().to_table_case(),
            field_name.to_string().to_table_case(),
            config.table_name,
            foreign_key.to_string()
        );
        let struct_name = &belongs_to.struct_name;
        let builder_field = format_ident!("{}_builder", &field_name);
        let child_builder_name = format_ident!("{}Builder", &struct_name.to_string());

        result.belongs_to_use.push(quote! {
            use super::#module_name::{#struct_name, #child_builder_name};
        });
        result
            .belongs_to_field
            .push(quote! { pub #field_name: Option<#struct_name>, });
        result.belongs_to_init.push(quote! { #field_name: None, });
        result
            .belongs_to_builder_field
            .push(quote! { pub #builder_field: Option<Box<#child_builder_name>>, });
        result.belongs_to_builder_impl.push(quote! {
            pub fn #field_name<F>(&self, f: F) -> #self_builder_name
            where F: FnOnce(&#child_builder_name) -> #child_builder_name {
                #self_builder_name {
                    #builder_field: Some(
                        Box::new(f(self.#builder_field.as_ref().unwrap_or(&Default::default())))
                    ),
                    ..self.clone()
                }
            }
        });
        result.belongs_to_filters_impl.push(quote! {
                if let Some(builder) = &self.#builder_field {
                    result.append(&mut builder.filters());
                }
        });
        result.belongs_to_join.push(quote! {
            if let Some(builder) = &self.#builder_field {
                join_parts.push(#join.to_string());
                builder.join(join_parts);
            }
        });
        result.belongs_to_preload.push(quote! {
            if let Some(builder) = &self.#builder_field {
                if builder.preload {
                    let ids = result.iter().map(|x| x.#foreign_key).collect::<Vec<_>>();
                    let parents_builder = #struct_name::select().id().eq_any(ids);
                    let parents_builder = #child_builder_name {
                        from: parents_builder.from,
                        filters: parents_builder.filters,
                        ..(**builder).clone()
                    };
                    let parents = parents_builder.load(client).await?;
                    result.iter_mut().for_each(|x| {
                        for parent in parents.iter() {
                            if x.#foreign_key == parent.id {
                                x.#field_name = Some(parent.clone());
                                break;
                            }
                        }
                    });
                }
            }
        });
    }
    result
}
