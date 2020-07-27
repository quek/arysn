use crate::generator::config::Config;
use inflector::Inflector;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

#[derive(Default)]
pub struct HasOne {
    pub has_one_use_plain: Vec<Vec<TokenStream>>,
    pub has_one_use_impl: Vec<Vec<TokenStream>>,
    pub has_one_field: Vec<TokenStream>,
    pub has_one_init: Vec<TokenStream>,
    pub has_one_builder_field: Vec<TokenStream>,
    pub has_one_builder_impl: Vec<TokenStream>,
    pub has_one_filters_impl: Vec<TokenStream>,
    pub has_one_join: Vec<TokenStream>,
    pub has_one_preload: Vec<TokenStream>,
}

pub fn make_has_one(config: &Config, self_builder_name: &Ident) -> HasOne {
    let mut result: HasOne = HasOne::default();
    for has_one in config.has_one.iter() {
        let module_ident = format_ident!("{}", has_one.struct_name.to_table_case().to_singular());
        let module_impl_ident = format_ident!("{}_impl", module_ident);
        let field_ident = format_ident!("{}", has_one.field);
        let foreign_key_ident = format_ident!("{}", has_one.foreign_key);
        let child_table_name = has_one.struct_name.to_table_case();
        let child_table_name_as = if has_one.field.to_table_case() != child_table_name {
            has_one.field
        } else {
            &child_table_name
        };
        let join_as = if child_table_name == child_table_name_as {
            "".to_string()
        } else {
            format!(" AS {}", child_table_name_as)
        };
        // TODO config.table_name は join as が連鎖している場合動かないと思う。動的にする。
        let join = format!(
            "INNER JOIN {}{} ON {}.{} = {}.id",
            child_table_name, join_as, child_table_name_as, has_one.foreign_key, config.table_name,
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
                    join_parts.push(#join.to_string());
                    builder.join(join_parts);
                }
            }
        });
        result.has_one_preload.push(quote! {
            if let Some(builder) = &self.#builder_field {
                if builder.preload {
                    let ids = result.iter().map(|x| x.id).collect::<Vec<_>>();
                    let children_builder = #struct_ident::select().#foreign_key_ident().eq_any(ids);
                    let children_builder = #child_builder_ident {
                        from: children_builder.from,
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
                    let children = children_builder.load(client).await?;
                    result.iter_mut().for_each(|x| {
                        for child in children.iter() {
                            if x.id == child.#foreign_key_ident {
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
