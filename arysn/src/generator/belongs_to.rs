use crate::generator::config::Config;
use inflector::Inflector;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

#[derive(Default)]
pub struct BelongsTo {
    pub belongs_to_use_plain: Vec<TokenStream>,
    pub belongs_to_use_impl: Vec<TokenStream>,
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
        let module_ident =
            format_ident!("{}", belongs_to.struct_name.to_table_case().to_singular());
        let module_impl_ident = format_ident!("{}_impl", module_ident);
        let field_ident = format_ident!("{}", belongs_to.field);
        let foreign_key_ident = format_ident!("{}", belongs_to.foreign_key);
        let parent_table_name = belongs_to.struct_name.to_table_case();
        let parent_table_name_as = if belongs_to.field.to_table_case() != parent_table_name {
            belongs_to.field
        } else {
            &parent_table_name
        };
        let join_as = if parent_table_name == parent_table_name_as {
            "".to_string()
        } else {
            format!(" AS {}", parent_table_name_as)
        };
        let join = format!(
            "INNER JOIN {}{} ON {}.id = {}.{}",
            parent_table_name,
            join_as,
            parent_table_name_as,
            config.table_name,
            foreign_key_ident.to_string()
        );
        let struct_ident = format_ident!("{}", belongs_to.struct_name);
        let builder_field = format_ident!("{}_builder", field_ident);
        let child_builder_ident = format_ident!("{}Builder", &struct_ident.to_string());

        result.belongs_to_use_plain.push(quote! {
            use super::#module_ident::#struct_ident;
        });
        result.belongs_to_use_impl.push(quote! {
            use super::#module_ident::#struct_ident;
            use super::#module_impl_ident::#child_builder_ident;
        });
        result
            .belongs_to_field
            .push(quote! { pub #field_ident: Option<#struct_ident>, });
        result.belongs_to_init.push(quote! { #field_ident: None, });
        result
            .belongs_to_builder_field
            .push(quote! { pub #builder_field: Option<Box<#child_builder_ident>>, });
        result.belongs_to_builder_impl.push(quote! {
            pub fn #field_ident<F>(&self, f: F) -> #self_builder_name
            where F: FnOnce(&#child_builder_ident) -> #child_builder_ident {
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
                    let ids = result.iter().map(|x| x.#foreign_key_ident).collect::<Vec<_>>();
                    let parents_builder = #struct_ident::select().id().eq_any(ids);
                    let parents_builder = #child_builder_ident {
                        from: parents_builder.from,
                        filters: parents_builder.filters,
                        ..(**builder).clone()
                    };
                    let parents = parents_builder.load(client).await?;
                    result.iter_mut().for_each(|x| {
                        for parent in parents.iter() {
                            if x.#foreign_key_ident == parent.id {
                                x.#field_ident = Some(parent.clone());
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
