use proc_macro2::TokenStream;
use quote::quote;
use syn_solidity::{File, Type};

use crate::{pp::PPFile, scope::Scope, types::get_sol_storage_type};

mod contract;
pub use contract::ContractExpansion;

mod enumm;
pub use enumm::EnumExpansion;

mod file;
pub use file::FileExpansion;

mod structure;
pub use structure::StructureExpansion;

mod udt;
pub use udt::UdtExpansion;

pub fn expand_storage(file: File) -> syn::Result<TokenStream> {
    let pp_file = PPFile::pre_process(&file)?;
    let scope = Scope::top_level(&pp_file);
    let expansion = FileExpansion::expand(&scope, &pp_file)?;
    Ok(expansion.into_tokens())
}

pub fn expand_type(ty: Type) -> syn::Result<TokenStream> {
    get_sol_storage_type(&quote! { ::sdecode_solidity::sol_types }, None, &ty)
}

pub fn expand_type_with_path(sol_types: TokenStream, ty: Type) -> syn::Result<TokenStream> {
    get_sol_storage_type(&sol_types, None, &ty)
}
