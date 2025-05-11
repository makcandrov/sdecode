use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;

use crate::{
    get_sol_storage_type,
    pp::PPUdt,
    scope::Scope,
    types::{get_default_rust_type, is_mapping_key_type},
};

#[derive(Debug, Clone, Default)]
pub struct UdtExpansion {
    pub structure_def: TokenStream,
    pub sol_storage_type_impl: TokenStream,
    pub sol_storage_value_impl: TokenStream,
    pub sol_mapping_key_type_impl: TokenStream,
}

impl UdtExpansion {
    pub fn expand(sc: &Scope<'_>, udt: &PPUdt<'_>) -> syn::Result<Self> {
        Ok(Self {
            structure_def: expand_structure_def(sc, udt)?,
            sol_storage_type_impl: expand_sol_storage_type_impl(sc, udt)?,
            sol_storage_value_impl: expand_sol_storage_value_impl(sc, udt)?,
            sol_mapping_key_type_impl: expand_sol_mapping_key_type_impl(sc, udt)?,
        })
    }

    pub fn into_tokens(self) -> TokenStream {
        let mut res = TokenStream::new();
        res.extend(self.structure_def);
        res.extend(self.sol_storage_type_impl);
        res.extend(self.sol_storage_value_impl);
        res.extend(self.sol_mapping_key_type_impl);
        res
    }
}

fn expand_structure_def(sc: &Scope<'_>, udt: &PPUdt<'_>) -> syn::Result<TokenStream> {
    if udt.attrs.remote.is_some() {
        return Ok(TokenStream::new());
    }

    let structure_ident = udt.rust_ident();
    let remaining_attrs = &udt.remaining_attrs;

    let typ = if let Some(overriden_type) = &udt.attrs.typ {
        quote! { #overriden_type }
    } else {
        get_default_rust_type(sc, &udt.raw.ty)?
    };

    let structure_def = quote! {
        #remaining_attrs
        pub struct #structure_ident (pub #typ);
    };

    Ok(structure_def)
}

fn expand_sol_storage_type_impl(sc: &Scope<'_>, udt: &PPUdt<'_>) -> syn::Result<TokenStream> {
    let udt_path = udt.rust_path();
    let udt_ident = udt.rust_ident();
    let udt_ident_lit = LitStr::new(&udt_ident.to_string(), udt_ident.span());

    let sdecode_solidity = sc.file.sdecode_solidity();

    let sol_storage_type_impl = quote! {
        #[automatically_derived]
        #[allow(
            non_camel_case_types,
            non_snake_case,
            clippy::pub_underscore_fields,
            clippy::style
        )]
        impl #sdecode_solidity::SolStorageType for #udt_path {
            const SOL_STORAGE_NAME: &str = #udt_ident_lit;
        }
    };

    Ok(sol_storage_type_impl)
}

fn expand_sol_storage_value_impl(sc: &Scope<'_>, udt: &PPUdt<'_>) -> syn::Result<TokenStream> {
    let udt_path = udt.rust_path();
    let sol_typ = get_sol_storage_type(sc, &udt.raw.ty)?;

    let sdecode_solidity = sc.file.sdecode_solidity();
    let sdecode_core = sc.file.sdecode_core();

    let sol_storage_value_impl = quote! {
        #[automatically_derived]
        #[allow(
            non_camel_case_types,
            non_snake_case,
            clippy::pub_underscore_fields,
            clippy::style
        )]
        impl #sdecode_solidity::SolStorageValue<Self> for #udt_path {
            fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, #sdecode_solidity::SolLayoutError>
            where
                Reader: #sdecode_core::StorageReader,
            {
                let value = <
                    #sdecode_solidity::helpers::SolStructureHelper<Self, _, #sol_typ>
                    as #sdecode_solidity::SolStorageValue<_>
                >::decode_storage(storage_reader)?.0;

                Ok(Self(value))
            }
        }
    };

    Ok(sol_storage_value_impl)
}

fn expand_sol_mapping_key_type_impl(sc: &Scope<'_>, udt: &PPUdt<'_>) -> syn::Result<TokenStream> {
    let udt_path = udt.rust_path();

    let typ = if let Some(overriden_type) = &udt.attrs.typ {
        quote! { #overriden_type }
    } else {
        get_default_rust_type(sc, &udt.raw.ty)?
    };
    let sol_typ = get_sol_storage_type(sc, &udt.raw.ty)?;

    let sdecode_solidity = sc.file.sdecode_solidity();
    let alloy_primitives = sc.file.alloy_primitives();

    let sol_mapping_key_type_impl = if is_mapping_key_type(sc, &udt.raw.ty)? {
        quote! {
            #[automatically_derived]
            impl #sdecode_solidity::SolMappingKeyType for #udt_path {}

            #[automatically_derived]
            impl #sdecode_solidity::SolMappingKeyValue<Self> for #udt_path {
                fn into_sol_mapping_key(self) -> #alloy_primitives::Bytes {
                    <
                        #typ as #sdecode_solidity::SolMappingKeyValue<#sol_typ>
                    >::into_sol_mapping_key(self.0)
                }

                fn try_from_sol_mapping_key(key: #alloy_primitives::Bytes) -> ::core::result::Result<Self, #alloy_primitives::Bytes> {
                    <
                        #typ as #sdecode_solidity::SolMappingKeyValue<#sol_typ>
                    >::try_from_sol_mapping_key(key).map(Self)
                }
            }
        }
    } else {
        TokenStream::new()
    };

    Ok(sol_mapping_key_type_impl)
}
