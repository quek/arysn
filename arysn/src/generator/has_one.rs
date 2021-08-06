use crate::generator::config::Config;
use crate::generator::Column;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashMap;

#[derive(Default)]
pub struct HasOne {
    pub has_one_use_plain: Vec<Vec<TokenStream>>,
    pub has_one_use_impl: Vec<Vec<TokenStream>>,
    pub has_one_field: Vec<TokenStream>,
    pub has_one_reader: Vec<TokenStream>,
    pub has_one_init: Vec<TokenStream>,
    pub has_one_builder_field: Vec<TokenStream>,
    pub has_one_builder_impl: Vec<TokenStream>,
    pub has_one_filters_impl: Vec<TokenStream>,
    pub has_one_join: Vec<TokenStream>,
    pub has_one_preload: Vec<TokenStream>,
}

pub fn make_has_one(
    config: &Config,
    self_builder_name: &Ident,
    columns_map: &HashMap<String, Vec<Column>>,
    configs: &[Config],
) -> HasOne {
    let mut result: HasOne = HasOne::default();
    let table_name = config.table_name;
    for has_one in config.has_one.iter() {
        let child_config = configs
            .iter()
            .find(|x| x.struct_name == has_one.struct_name)
            .unwrap();
        let module_ident = format_ident!("{}", child_config.mod_name());
        let module_impl_ident = format_ident!("{}_impl", module_ident);
        let field_ident = format_ident!("{}", has_one.field);
        let foreign_key_ident = format_ident!("{}", has_one.foreign_key);
        let child_table_name = child_config.table_name;
        let child_table_name_as = if has_one.field != child_config.mod_name() {
            has_one.field
        } else {
            &child_table_name
        };
        let join_as = if child_table_name == child_table_name_as {
            "".to_string()
        } else {
            format!(" AS {}", child_table_name_as)
        };
        let join = format!(
            "{{}} JOIN {}{} ON {}.{} = {{}}.id",
            child_table_name, join_as, child_table_name_as, has_one.foreign_key,
        );
        let struct_ident = format_ident!("{}", has_one.struct_name);
        let builder_field = format_ident!("{}_builder", field_ident.to_string());
        let child_builder_ident = format_ident!("{}Builder", &struct_ident.to_string());

        result
            .has_one_use_plain
            .push(vec![quote! ( use super::#module_ident::#struct_ident; )]);
        result.has_one_use_impl.push(vec![
            quote! ( use super::#module_ident::#struct_ident; ),
            quote! ( use super::#module_impl_ident::#child_builder_ident; ),
        ]);
        result
            .has_one_field
            .push(quote! { pub #field_ident: Option<Box<#struct_ident>>, });
        result.has_one_reader.push(quote! {
            pub fn #field_ident(&self) -> &#struct_ident {
                self.#field_ident.as_ref().unwrap()
            }
        });
        result.has_one_init.push(quote! { #field_ident: None, });
        result
            .has_one_builder_field
            .push(quote! { pub #builder_field: Option<Box<#child_builder_ident>>, });
        result.has_one_builder_impl.push(quote! {
            pub fn #field_ident<F>(&self, f: F) -> #self_builder_name
            where F: FnOnce(&#child_builder_ident) -> #child_builder_ident {
                let child_builder = f(self.#builder_field.as_ref().unwrap_or(
                    &Box::new(#child_builder_ident {
                        table_name_as: Some(#child_table_name_as.to_string()),
                        ..Default::default()
                    })
                ));
                let mut builder = self.clone();
                builder.#builder_field = Some(Box::new(child_builder));
                builder
            }
        });
        result.has_one_filters_impl.push(quote! {
            if let Some(builder) = &self.#builder_field {
                result.append(&mut builder.filters());
            }
        });
        result.has_one_join.push(quote! {
            if let Some(builder) = &self.#builder_field {
                if !builder.filters().is_empty() {
                    join_parts.push(
                        format!(#join,
                                if builder.outer_join { "LEFT OUTER" } else { "INNER" },
                                self.table_name_as.as_ref().unwrap_or(&#table_name.to_string()))
                    );
                    builder.join(join_parts);
                }
            }
        });
        let column = columns_map[child_table_name]
            .iter()
            .find(|column| column.name == has_one.foreign_key)
            .unwrap();
        let foreign_key_value = if column.is_nullable {
            quote! { child.#foreign_key_ident.unwrap() }
        } else {
            quote! { child.#foreign_key_ident }
        };
        result.has_one_preload.push(quote! {
            if let Some(builder) = &self.#builder_field {
                if builder.preload {
                    let ids = result.iter().map(|x| x.id).collect::<Vec<_>>();
                    let children_builder = #struct_ident::select().#foreign_key_ident().r#in(ids);
                    let children_builder = #child_builder_ident {
                        from: children_builder.from,
                        table_name_as: None,
                        filters: builder.filters.iter().cloned()
                            .chain(children_builder.filters.into_iter())
                            .map(|x| Filter {
                                table: #child_table_name.to_string(),
                                preload: false,
                                ..x
                            })
                            .collect::<Vec<_>>(),
                        ..(**builder).clone()
                    };
                    let children = children_builder.load(conn).await?;
                    result.iter_mut().for_each(|x| {
                        for child in children.iter() {
                            if x.id == #foreign_key_value {
                                x.#field_ident = Some(Box::new(child.clone()));
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
