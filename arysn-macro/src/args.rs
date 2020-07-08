use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::braced;
use syn::parse::{Parse, ParseStream};
use syn::Token;

pub struct Args {
    pub struct_name: Ident,
    pub _brace_token: syn::token::Brace,
    pub fields: syn::punctuated::Punctuated<syn::Field, Token![,]>,
}

impl Args {
    pub fn get(&self, key: &str) -> Option<TokenStream> {
        self.fields
            .iter()
            .find(|field| {
                field
                    .ident
                    .as_ref()
                    .map(|x| x.to_string().as_str() == key)
                    .unwrap_or(false)
            })
            .map(|field| {
                let ty = &field.ty;
                let x = quote! { #ty };
                x
            })
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> std::result::Result<Self, syn::Error> {
        let content;
        Ok(Self {
            struct_name: input.parse()?,
            _brace_token: braced!(content in input),
            fields: content.parse_terminated(syn::Field::parse_named)?,
        })
    }
}
