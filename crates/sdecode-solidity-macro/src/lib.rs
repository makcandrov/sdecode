#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use proc_macro2::TokenStream;
use syn::parse_macro_input;
use syn_solidity::File;

mod array_size;

mod attribute;

mod expansion;
use expansion::FileExpansion;

mod linearize;

mod pp;
use pp::PPFile;

mod scope;
use scope::Scope;

mod utils;

mod types;
use types::get_sol_storage_type;

#[proc_macro]
pub fn sol_storage(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as File);

    match expand(input) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error(),
    }
    .into()
}

fn expand(file: File) -> syn::Result<TokenStream> {
    let pp_file = PPFile::pre_process(&file)?;
    let scope = Scope::top_level(&pp_file);
    let expansion = FileExpansion::expand(&scope, &pp_file)?;
    Ok(expansion.into_tokens())
}
