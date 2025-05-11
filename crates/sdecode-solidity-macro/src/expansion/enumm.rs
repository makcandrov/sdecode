use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::LitStr;
use syn_solidity::Spanned;

use crate::{attribute::attrs_tokens, pp::PPEnum, scope::Scope};

#[derive(Debug, Clone, Default)]
pub struct EnumExpansion {
    pub enum_def: TokenStream,
    pub try_from_u8_impl: TokenStream,
    pub sol_storage_type_impl: TokenStream,
    pub sol_storage_value_impl: TokenStream,
}

impl EnumExpansion {
    pub fn expand<'f>(sc: &Scope<'f>, enumm: &'f PPEnum<'_>) -> syn::Result<Self> {
        Ok(Self {
            enum_def: expand_enum_def(sc, enumm)?,
            try_from_u8_impl: expand_try_from_u8_impl(sc, enumm)?,
            sol_storage_type_impl: expand_sol_storage_type_impl(sc, enumm)?,
            sol_storage_value_impl: expand_sol_storage_value_impl(sc, enumm)?,
        })
    }

    pub fn into_tokens(self) -> TokenStream {
        let mut res = TokenStream::new();
        res.extend(self.enum_def);
        res.extend(self.try_from_u8_impl);
        res.extend(self.sol_storage_type_impl);
        res.extend(self.sol_storage_value_impl);
        res
    }
}

fn expand_enum_def(_sc: &Scope<'_>, enumm: &PPEnum<'_>) -> syn::Result<TokenStream> {
    if enumm.attrs.remote.is_some() {
        return Ok(TokenStream::default());
    }

    let mut enum_variants = TokenStream::new();

    for variant in &enumm.raw.variants {
        let remaining_attrs = attrs_tokens(&variant.attrs);

        let variant_name = &variant.ident;

        enum_variants.extend(quote! {
            #remaining_attrs
            #variant_name,
        });
    }

    let enum_ident = enumm.rust_path();

    let remaining_attrs = &enumm.remaining_attrs;

    let enum_def = quote! {
        #remaining_attrs
        pub enum #enum_ident {
            #enum_variants
        }
    };

    Ok(enum_def)
}

fn expand_try_from_u8_impl(_sc: &Scope<'_>, enumm: &PPEnum<'_>) -> syn::Result<TokenStream> {
    if enumm.attrs.remote.is_some() {
        return Ok(TokenStream::default());
    }

    let mut try_from_u8_match = TokenStream::new();

    for (i, variant) in enumm.raw.variants.iter().enumerate() {
        let Ok(i) = u8::try_from(i) else {
            return Err(syn::Error::new(variant.span(), "too many variants"));
        };

        let variant_name = &variant.ident;

        let index_lit = Literal::u8_unsuffixed(i);
        try_from_u8_match.extend(quote! {
            #index_lit => Ok(Self::#variant_name),
        });
    }

    let enum_ident = enumm.rust_path();

    let try_from_u8_impl = quote! {
        #[automatically_derived]
        #[allow(
            non_camel_case_types,
            non_snake_case,
            clippy::pub_underscore_fields,
            clippy::style
        )]
        impl ::core::convert::TryFrom<u8> for #enum_ident {
            type Error = u8;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    #try_from_u8_match
                    value => Err(value),
                }
            }
        }
    };

    Ok(try_from_u8_impl)
}

fn expand_sol_storage_type_impl(sc: &Scope<'_>, enumm: &PPEnum<'_>) -> syn::Result<TokenStream> {
    let enum_path = enumm.rust_path();
    let enum_ident = enumm.rust_ident();
    let enum_ident_lit = LitStr::new(&enum_ident.to_string(), enum_ident.span());

    let sdecode_solidity = sc.file.sdecode_solidity();

    let sol_storage_type_impl = quote! {
        #[automatically_derived]
        #[allow(
            non_camel_case_types,
            non_snake_case,
            clippy::pub_underscore_fields,
            clippy::style
        )]
        impl #sdecode_solidity ::SolStorageType for #enum_path {
            const SOL_STORAGE_NAME: &str = #enum_ident_lit;
        }
    };

    Ok(sol_storage_type_impl)
}

fn expand_sol_storage_value_impl(sc: &Scope<'_>, enumm: &PPEnum<'_>) -> syn::Result<TokenStream> {
    let enum_ident = enumm.rust_ident();

    let sdecode_core = sc.file.sdecode_core();
    let sdecode_solidity = sc.file.sdecode_solidity();

    let sol_storage_value_impl = quote! {
        #[automatically_derived]
        impl #sdecode_solidity ::SolStorageValue<Self> for #enum_ident {
            fn decode_storage<Reader>(storage_reader: &mut Reader) -> ::core::result::Result<Self, #sdecode_solidity::SolLayoutError>
            where
                Reader: #sdecode_core::StorageReader,
            {
                let res = <
                    #sdecode_solidity::helpers::SolEnumHelper<Self, Self>
                    as #sdecode_solidity::SolStorageValue<_>
                >::decode_storage(storage_reader)?.0;

                Ok(res)
            }
        }
    };

    Ok(sol_storage_value_impl)
}
