#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use syn::parse_macro_input;
use syn_solidity::{File, Type};

mod array_size;
mod attribute;
mod case;
mod expansion;
mod linearize;
mod pp;
mod scope;
mod types;

#[proc_macro]
pub fn sol_storage(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let file = parse_macro_input!(input as File);
    expansion::expand_storage(file)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn sol_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ty = parse_macro_input!(input as Type);
    expansion::expand_type(ty)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn sol_type_with_path(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    struct TypeWithPath {
        sol_types: syn::Path,
        ty: Type,
    }

    impl syn::parse::Parse for TypeWithPath {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let content;
            syn::bracketed!(content in input);
            let sol_types = content.parse::<syn::Path>()?;
            let ty = input.parse::<Type>()?;
            Ok(Self { sol_types, ty })
        }
    }

    let TypeWithPath { sol_types, ty } = parse_macro_input!(input as TypeWithPath);
    expansion::expand_type_with_path(quote::quote! { #sol_types }, ty)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
