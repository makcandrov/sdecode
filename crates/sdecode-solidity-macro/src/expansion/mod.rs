use proc_macro2::TokenStream;
use syn_solidity::File;

use crate::{pp::PPFile, scope::Scope};

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

pub fn expand(file: File) -> syn::Result<TokenStream> {
    let pp_file = PPFile::pre_process(&file)?;
    let scope = Scope::top_level(&pp_file);
    let expansion = FileExpansion::expand(&scope, &pp_file)?;
    Ok(expansion.into_tokens())
}
