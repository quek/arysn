extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::Ident;

#[proc_macro]
pub fn defar(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as Args);
    println!("args {:?}", &args);
    let name = &args.name;
    let gen = quote! {
        #[derive(Debug)]
        struct #name {
            pub name: String,
        }
    };
    gen.into()
}

#[derive(Debug)]
struct Args {
    name: Ident,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            name: input.parse()?,
        })
    }
}
