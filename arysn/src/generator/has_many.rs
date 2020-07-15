use crate::generator::config::Config;
use inflector::Inflector;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

#[derive(Default)]
pub struct HasMany {
    pub has_many_use: Vec<TokenStream>,
    pub has_many_field: Vec<TokenStream>,
    pub has_many_init: Vec<TokenStream>,
    pub has_many_builder_field: Vec<TokenStream>,
    pub has_many_builder_impl: Vec<TokenStream>,
    pub has_many_filters_impl: Vec<TokenStream>,
    pub has_many_join: Vec<TokenStream>,
    pub has_many_preload: Vec<TokenStream>,
}

pub fn make_has_many(config: &Config, self_builder_name: &Ident) -> HasMany {
    let mut result: HasMany = HasMany::default();
    for has_many in config.has_many.iter() {
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

        result.has_many_use.push(quote! {
            use super::#module_name::{#struct_name, #child_builder_name};
        });
        result
            .has_many_field
            .push(quote! { pub #field_name: Option<Vec<#struct_name>>, });
        result.has_many_init.push(quote! { #field_name: None, });
        result
            .has_many_builder_field
            .push(quote! { pub #builder_field: Option<Box<#child_builder_name>>, });
        result.has_many_builder_impl.push(quote! {
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
        result.has_many_filters_impl.push(quote! {
            if let Some(builder) = &self.#builder_field {
                result.append(&mut builder.filters());
            }
        });
        result.has_many_join.push(quote! {
            if let Some(builder) = &self.#builder_field {
                join_parts.push(#join.to_string());
                builder.join(join_parts);
            }
        });
        result.has_many_preload.push(quote! {
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
        });
    }
    result
}
