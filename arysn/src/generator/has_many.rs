use crate::generator::config::Config;
use crate::generator::Column;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashMap;

#[derive(Default)]
pub struct HasMany {
    pub has_many_use_plain: Vec<Vec<TokenStream>>,
    pub has_many_use_impl: Vec<Vec<TokenStream>>,
    pub has_many_field: Vec<TokenStream>,
    pub has_many_init: Vec<TokenStream>,
    pub has_many_builder_impl: Vec<TokenStream>,
    pub has_many_preload: Vec<TokenStream>,
}

pub fn make_has_many(
    config: &Config,
    self_builder_name: &Ident,
    columns_map: &HashMap<String, Vec<Column>>,
    configs: &[Config],
) -> HasMany {
    let mut result: HasMany = HasMany::default();
    let _table_name = config.table_name;
    for has_many in config.has_many.iter() {
        let child_config = configs
            .iter()
            .find(|x| x.struct_name == has_many.struct_name)
            .unwrap();
        let module_ident = format_ident!("{}", child_config.mod_name());
        let module_impl_ident = format_ident!("{}_impl", module_ident);
        let field_ident = format_ident!("{}", has_many.field);
        let foreign_key_ident = format_ident!("{}", has_many.foreign_key);
        let child_table_name = child_config.table_name;
        let child_table_name_as = if has_many.field != child_config.mod_name() {
            has_many.field
        } else {
            &child_table_name
        };
        let struct_ident = format_ident!("{}", has_many.struct_name);
        let child_builder_ident = format_ident!("{}Builder", &struct_ident.to_string());

        if config.struct_name != has_many.struct_name {
            result
                .has_many_use_plain
                .push(vec![quote! ( use super::#module_ident::#struct_ident; )]);
            result.has_many_use_impl.push(vec![
                quote! ( use super::#module_ident::#struct_ident; ),
                quote! ( use super::#module_impl_ident::#child_builder_ident; ),
            ]);
        }
        result
            .has_many_field
            .push(quote! { pub #field_ident: Vec<#struct_ident>, });
        result.has_many_init.push(quote! { #field_ident: vec![], });
        result.has_many_builder_impl.push(quote! {
            pub fn #field_ident<F>(&self, f: F) -> #self_builder_name
            where F: FnOnce(&#child_builder_ident) -> #child_builder_ident {
                let mut builder = self.clone();
                builder
                    .filters
                    .push(Filter::Builder(Box::new(f(&#child_builder_ident {
                        from: #child_table_name.to_string(),
                        table_name_as: Some(#child_table_name_as.to_string()),
                        relation_type: RelationType::HasMany(stringify!(#foreign_key_ident)),
                        ..Default::default()
                    }))));
                builder
            }
        });
        let column = columns_map[child_table_name]
            .iter()
            .find(|column| column.name == has_many.foreign_key)
            .unwrap();
        let foreign_key_value = if column.is_nullable {
            quote! { child.#foreign_key_ident.unwrap() }
        } else {
            quote! { child.#foreign_key_ident }
        };
        result.has_many_preload.push(quote! {
            let builders = self.filters.iter().filter_map(|filter| match filter {
                Filter::Builder(builder) if builder.table_name_as_or() == #child_table_name_as => Some(builder),
                _ => None,
            }).collect::<Vec<_>>();
            if builders.iter().any(|x| x.preload()) {
                let ids = result.iter().map(|x| x.id).collect::<Vec<_>>();
                let mut children_builder = #struct_ident::select().#foreign_key_ident().r#in(ids);
                for builder in &builders {
                    children_builder.filters.append(&mut builder.filters().clone());
                    children_builder.orders.append(&mut builder.order().clone());
                    if builder.group_by().is_some() {
                        children_builder.group_by = builder.group_by();
                    }
                    if builder.limit().is_some() {
                        children_builder.limit = builder.limit();
                    }
                    if builder.offset().is_some() {
                        children_builder.offset = builder.offset();
                    }
                }
                for filter in children_builder.filters.iter_mut() {
                    match filter {
                        Filter::Column(column) => {
                            column.table = #child_table_name.to_string();
                            column.preload = false;
                        }
                        _ => ()
                    }
                }
                let children = children_builder.load(conn).await?;
                result.iter_mut().for_each(|x| {
                    let mut ys = vec![];
                    for child in children.iter() {
                        if x.id == #foreign_key_value {
                            ys.push(child.clone());
                        }
                    }
                    x.#field_ident = ys;
                });
            }
        });
    }
    result
}
