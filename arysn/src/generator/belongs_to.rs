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
    pub belongs_to_builder_field: Vec<TokenStream>,
    pub belongs_to_builder_impl: Vec<TokenStream>,
    pub belongs_to_filters_impl: Vec<TokenStream>,
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
        let builder_field = format_ident!("{}_builder", field_ident);
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
                match &self.#builder_field.as_ref().map(|x| x.table_name_as.as_ref()).flatten() {
                    Some(table_name_as) => format!(
                        #y,
                        if builder.outer_join { "LEFT OUTER" } else { "INNER" },
                        table_name_as,
                        #parent_table_name
                    ),
                    None => format!(
                        #x,
                        if builder.outer_join { "LEFT OUTER" } else { "INNER" },
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
        result
            .belongs_to_builder_field
            .push(quote! { pub #builder_field: Option<Box<#parent_builder_ident>>, });
        result.belongs_to_builder_impl.push(quote! {
            pub fn #field_ident<F>(&self, f: F) -> #self_builder_name
            where F: FnOnce(&#parent_builder_ident) -> #parent_builder_ident {
                let parent_builder = f(self.#builder_field.as_ref().unwrap_or(
                    &Box::new(#parent_builder_ident {
                        table_name_as: Some(#parent_table_name_as.to_string()),
                        ..Default::default()
                    })
                ));
                let mut builder = self.clone();
                builder.#builder_field = Some(Box::new(parent_builder));
                builder
            }
        });
        result.belongs_to_filters_impl.push(quote! {
                if let Some(builder) = &self.#builder_field {
                    result.append(&mut builder.filters());
                }
        });
        result.belongs_to_join.push(quote! {
            if let Some(builder) = &self.#builder_field {
                if !builder.filters().is_empty() {
                    join_parts.push(#join);
                    builder.join(join_parts);
                }
            }
        });
        result.belongs_to_preload.push({
            let (map, parent_id) = if column.is_nullable {
                (quote!(filter_map), quote!(Some(parent.id)))
            } else {
                (quote!(map), quote!(parent.id))
            };
            quote! {
                if let Some(builder) = &self.#builder_field {
                    if builder.preload {
                        let ids = result.iter().#map(|x| x.#foreign_key_ident).collect::<std::collections::HashSet<_>>();
                        let ids = ids.into_iter().collect::<Vec<_>>();
                        let parents_builder = #struct_ident::select().id().r#in(ids);
                        let parents_builder = #parent_builder_ident {
                            from: parents_builder.from,
                            table_name_as: None,
                            filters: builder.filters.iter().cloned()
                                .chain(parents_builder.filters.into_iter())
                                .map(|x| Filter {
                                    table: #parent_table_name.to_string(),
                                    preload: false,
                                    ..x
                                })
                                .collect::<Vec<_>>(),
                            ..(**builder).clone()
                        };
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
            }
        });
    }
    result
}
