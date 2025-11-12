#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use syn::parse_macro_input;
use syn_solidity::File;

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
    expansion::expand(file)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
