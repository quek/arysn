use crate::generator::config::Config;
use inflector::Inflector;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

#[derive(Default)]
pub struct HasMany {
    pub has_many_use_plain: Vec<Vec<TokenStream>>,
    pub has_many_use_impl: Vec<Vec<TokenStream>>,
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
    let table_name = config.table_name;
    for has_many in config.has_many.iter() {
        let module_ident = format_ident!("{}", has_many.struct_name.to_table_case().to_singular());
        let module_impl_ident = format_ident!("{}_impl", module_ident);
        let field_ident = format_ident!("{}", has_many.field);
        let foreign_key_ident = format_ident!("{}", has_many.foreign_key);
        let child_table_name = has_many.struct_name.to_table_case();
        let child_table_name_as = if has_many.field.to_table_case() != child_table_name {
            has_many.field
        } else {
            &child_table_name
        };
        let struct_ident = format_ident!("{}", has_many.struct_name);
        let builder_field = format_ident!("{}_builder", field_ident.to_string());
        let child_builder_ident = format_ident!("{}Builder", &struct_ident.to_string());
        let join = {
            let x = format!(
                "INNER JOIN {} ON {}.{} = {{}}.id",
                child_table_name, child_table_name_as, has_many.foreign_key,
            );
            let y = format!(
                "INNER JOIN {} AS {{0}} ON {{0}}.{} = {{1}}.id",
                child_table_name, has_many.foreign_key,
            );
            let parent_table_name = quote! {
                self.table_name_as.as_ref().unwrap_or(&#table_name.to_string())
            };
            quote! {
                match &self.#builder_field.as_ref().map(|x| x.table_name_as.as_ref()).flatten() {
                    Some(table_name_as) => format!(#y, table_name_as, #parent_table_name),
                    None => format!(#x, #parent_table_name)
                }
            }
        };

        result
            .has_many_use_plain
            .push(vec![quote! ( use super::#module_ident::#struct_ident; )]);
        result.has_many_use_impl.push(vec![
            quote! ( use super::#module_ident::#struct_ident; ),
            quote! ( use super::#module_impl_ident::#child_builder_ident; ),
        ]);
        result
            .has_many_field
            .push(quote! { pub #field_ident: Vec<#struct_ident>, });
        result.has_many_init.push(quote! { #field_ident: vec![], });
        result
            .has_many_builder_field
            .push(quote! { pub #builder_field: Option<Box<#child_builder_ident>>, });
        result.has_many_builder_impl.push(quote! {
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
        result.has_many_filters_impl.push(quote! {
            if let Some(builder) = &self.#builder_field {
                result.append(&mut builder.filters());
            }
        });
        result.has_many_join.push(quote! {
            if let Some(builder) = &self.#builder_field {
                if !builder.filters().is_empty() {
                    join_parts.push(#join);
                    builder.join(join_parts);
                }
            }
        });
        result.has_many_preload.push(quote! {
            if let Some(builder) = &self.#builder_field {
                if builder.preload {
                    let ids = result.iter().map(|x| x.id).collect::<Vec<_>>();
                    let children_builder = #struct_ident::select().#foreign_key_ident().r#in(ids);
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
                    let children = children_builder.load(conn).await?;
                    result.iter_mut().for_each(|x| {
                        let mut ys = vec![];
                        for child in children.iter() {
                            if x.id == child.#foreign_key_ident {
                                ys.push(child.clone());
                            }
                        }
                        x.#field_ident = ys;
                    });
                }
            }
        });
    }
    result
}
