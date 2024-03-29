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
                &belongs_to.foreign_key, table_name
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
                        relation_type: RelationType::BelongsTo(stringify!(#foreign_key_ident)),
                        ..Default::default()
                    }))));
                builder
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
                    Filter::Builder(builder) if builder.table_name_as_or() == #parent_table_name_as => Some(builder),
                    _ => None,
                }).collect::<Vec<_>>();
                if builders.iter().any(|x| x.preload()) {
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
                                column.table = #parent_table_name.to_string(); 
                                column.preload = false;
                            }
                            _ => ()
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
