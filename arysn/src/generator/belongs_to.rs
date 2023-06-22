use crate::generator::config::Config;
use crate::generator::Column;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

#[derive(Default)]
pub struct BelongsTo {
    pub belongs_to_use_plain: Vec<Vec<TokenStream>>,
    pub belongs_to_use_impl: Vec<Vec<TokenStream>>,
    pub belongs_to_field: Vec<TokenStream>,
    pub belongs_to_reader: Vec<TokenStream>,
    pub belongs_to_init: Vec<TokenStream>,
    pub belongs_to_builder_impl: Vec<TokenStream>,
    pub belongs_to_join: Vec<TokenStream>,
    pub belongs_to_preload: Vec<TokenStream>,
}

pub fn make_belongs_to(
    config: &Config,
    self_builder_name: &Ident,
    columns: &Vec<Column>,
    configs: &Vec<Config>,
) -> BelongsTo {
    let mut result: BelongsTo = BelongsTo::default();
    let table_name = config.table_name;
    for belongs_to in config.belongs_to.iter() {
        let column = columns
            .iter()
            .find(|column| column.name == belongs_to.foreign_key)
            .expect(&format!(
                "{} is not found in {}",
                &belongs_to.foreign_key, &config.table_name
            ));
        let parent_config = configs
            .iter()
            .find(|x| x.struct_name == belongs_to.struct_name)
            .unwrap();
        let module_ident = format_ident!("{}", parent_config.mod_name());
        let module_impl_ident = format_ident!("{}_impl", module_ident);
        let field_ident = format_ident!("{}", belongs_to.field);
        let foreign_key_ident = format_ident!("{}", belongs_to.foreign_key);
        let parent_table_name = parent_config.table_name;
        let parent_table_name_as = if belongs_to.field != parent_config.mod_name() {
            belongs_to.field
        } else {
            &parent_table_name
        };
        let struct_ident = format_ident!("{}", belongs_to.struct_name);
        let parent_builder_ident = format_ident!("{}Builder", struct_ident);
        let join = {
            let x = format!(
                "{{}} JOIN {} ON {}.id = {{}}.{}",
                parent_table_name,
                parent_table_name_as,
                foreign_key_ident.to_string()
            );
            let y = format!(
                "{{0}} JOIN {} AS {{1}} ON {{1}}.id = {{2}}.{}",
                parent_table_name,
                foreign_key_ident.to_string()
            );
            let parent_table_name = quote! {
                self.table_name_as.as_ref().unwrap_or(&#table_name.to_string())
            };
            quote! {
                match table_name_as {
                    Some(table_name_as) => format!(
                        #y,
                        if outer_join { "LEFT OUTER" } else { "INNER" },
                        table_name_as,
                        #parent_table_name
                    ),
                    None => format!(
                        #x,
                        if outer_join { "LEFT OUTER" } else { "INNER" },
                        #parent_table_name
                    )
                }
            }
        };

        result.belongs_to_use_plain.push(vec![quote! {
            use super::#module_ident::#struct_ident;
        }]);
        result.belongs_to_use_impl.push(vec![
            quote! ( use super::#module_ident::#struct_ident; ),
            quote! ( use super::#module_impl_ident::#parent_builder_ident; ),
        ]);
        result
            .belongs_to_field
            .push(quote! { pub #field_ident: Option<#struct_ident>, });
        result.belongs_to_reader.push(quote! {
            pub fn #field_ident(&self) -> &#struct_ident {
                self.#field_ident.as_ref().unwrap()
            }
        });
        result.belongs_to_init.push(quote! { #field_ident: None, });
        result.belongs_to_builder_impl.push(quote! {
            pub fn #field_ident<F>(&self, f: F) -> #self_builder_name
            where F: FnOnce(&#parent_builder_ident) -> #parent_builder_ident {
                let mut builder = self.clone();
                builder
                    .filters
                    .push(Filter::Builder(Box::new(f(&#parent_builder_ident {
                        from: #parent_table_name.to_string(),
                        table_name_as: Some(#parent_table_name_as.to_string()),
                        ..Default::default()
                    }))));
                builder
            }
        });
        result.belongs_to_join.push(quote! {
            let builders = self.filters.iter().filter_map(|filter| match filter {
                Filter::Builder(builder)
                    if builder.table_name_as_or() == #parent_table_name_as
                        && !builder.query_filters().is_empty() => Some(builder),
                _ => None,
            }).collect::<Vec<_>>();
            let mut table_name = None;
            let mut table_name_as = None;
            let mut outer_join = false;
            for builder in &builders {
                table_name = Some(builder.table_name());
                if let Some(x) = builder.table_name_as() {
                    table_name_as = Some(x);
                }
                if builder.outer_join() {
                    outer_join = true;
                }
            }
            if let Some(table_name) = table_name {
                join_parts.push(#join);
            }
            for builder in &builders {
                builder.join(join_parts);
            }
        });
        result.belongs_to_preload.push({
            let (map, parent_id) = if column.is_nullable {
                (quote!(filter_map), quote!(Some(parent.id)))
            } else {
                (quote!(map), quote!(parent.id))
            };
            quote! {
                let builders = self.filters.iter().filter_map(|filter| match filter {
                    Filter::Builder(builder) if builder.preload() && builder.table_name_as_or() == #parent_table_name_as => Some(builder),
                    _ => None,
                }).collect::<Vec<_>>();
                if !builders.is_empty() {
                    let ids = result.iter().#map(|x| x.#foreign_key_ident).collect::<std::collections::HashSet<_>>();
                    let ids = ids.into_iter().collect::<Vec<_>>();
                    let mut parents_builder = #struct_ident::select().id().r#in(ids);
                    for builder in &builders {
                        parents_builder.filters.append(&mut builder.filters().clone());
                        parents_builder.orders.append(&mut builder.order().clone());
                        if builder.group_by().is_some() {
                            parents_builder.group_by = builder.group_by();
                        }
                        if builder.limit().is_some() {
                            parents_builder.limit = builder.limit();
                        }
                        if builder.offset().is_some() {
                            parents_builder.offset = builder.offset();
                        }
                    }
                    for filter in parents_builder.filters.iter_mut() {
                        match filter {
                            Filter::Column(column) => {
                                column.preload = false;
                            }
                            Filter::Builder(builder) => {
                                builder.table_name_as_mut().take();
                            }
                        }
                    }
                    let parents = parents_builder.load(conn).await?;
                    result.iter_mut().for_each(|x| {
                        for parent in parents.iter() {
                            if x.#foreign_key_ident == #parent_id {
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
