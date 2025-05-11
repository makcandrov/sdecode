use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn_solidity::Spanned;

use crate::{
    get_sol_storage_type,
    pp::{PPContract, PPVariableDef, UserDefinedItem},
    scope::Scope,
    types::get_default_rust_type,
};

use super::{EnumExpansion, StructureExpansion, UdtExpansion};

#[derive(Debug, Clone, Default)]
pub struct ContractExpansion {
    pub mod_name: TokenStream,
    pub pub_use: TokenStream,
    pub items: TokenStream,
    pub storage_structure_def: TokenStream,
    pub storage_decode_impl: TokenStream,
}

impl ContractExpansion {
    pub fn expand<'f>(sc: &Scope<'f>, contract: &'f PPContract<'_>) -> syn::Result<Self> {
        let mut structures = Vec::new();
        let mut udts = Vec::new();
        let mut enums = Vec::new();

        for udi in contract.udis.values() {
            match udi {
                UserDefinedItem::Enum(enumm) => {
                    let expansion = EnumExpansion::expand(sc, enumm)?;
                    enums.push(expansion);
                }
                UserDefinedItem::Struct(structure) => {
                    let expansion = StructureExpansion::expand(sc, structure)?;
                    structures.push(expansion);
                }
                UserDefinedItem::Udt(udt) => {
                    let expansion = UdtExpansion::expand(sc, udt)?;
                    udts.push(expansion);
                }
                UserDefinedItem::Contract(_) => unreachable!(),
                _ => {}
            }
        }

        let mut items = TokenStream::new();
        for structure in structures {
            items.extend(structure.into_tokens());
        }
        for udt in udts {
            items.extend(udt.into_tokens());
        }
        for enumm in enums {
            items.extend(enumm.into_tokens());
        }

        let mut storage_vars = Vec::new();
        for contract in sc.file.parents_of(contract) {
            for var in contract
                .udis
                .values()
                .filter_map(UserDefinedItem::as_variable)
            {
                if var.raw.attributes.has_immutable() || var.raw.attributes.has_constant() {
                    continue;
                }
                storage_vars.push((contract, var));
            }
        }

        let mod_name = contract.mod_name().to_token_stream();
        let pub_use = if contract.attrs.remote.is_some()
            || contract.raw.is_interface()
            || contract.raw.is_library()
        {
            TokenStream::new()
        } else {
            let storage_structure_ident = contract.rust_path();
            quote! { pub use #mod_name:: #storage_structure_ident; }
        };

        Ok(Self {
            mod_name,
            pub_use,
            items,
            storage_structure_def: expand_storage_structure_def(sc, contract, &storage_vars)?,
            storage_decode_impl: expand_storage_decode_impl(sc, contract, &storage_vars)?,
        })
    }

    pub fn into_tokens(self) -> TokenStream {
        let mut res = TokenStream::new();
        res.extend(self.items);
        res.extend(self.storage_structure_def);
        res.extend(self.storage_decode_impl);

        let mod_name = self.mod_name;
        let pub_use = self.pub_use;

        quote! {
            pub mod #mod_name {
                #res
            }
            #pub_use
        }
    }
}

fn expand_storage_structure_def(
    sc: &Scope<'_>,
    contract: &PPContract<'_>,
    vars: &Vec<(&PPContract<'_>, &PPVariableDef<'_>)>,
) -> syn::Result<TokenStream> {
    if contract.raw.is_interface() || contract.raw.is_library() {
        return Ok(TokenStream::new());
    }

    if contract.attrs.remote.is_some() {
        return Ok(TokenStream::new());
    }

    let mut contract_storage_fields = TokenStream::new();

    for (contract, var) in vars {
        let field_name = &var.raw.name;
        let field_ty = if let Some(overriden_type) = &var.attrs.typ {
            quote! { #overriden_type }
        } else {
            get_default_rust_type(&sc.in_contract(contract), &var.raw.ty)?
        };

        let remaining_attrs = &var.remaining_attrs;

        contract_storage_fields.extend(quote! {
            #[allow(missing_docs)]
            #remaining_attrs
            pub #field_name: #field_ty,
        });
    }

    let storage_structure_ident = contract.rust_path();

    let remaining_attrs = &contract.remaining_attrs;

    let storage_structure_def = quote! {
        #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
        #remaining_attrs
        pub struct #storage_structure_ident {
            #contract_storage_fields
        }
    };

    Ok(storage_structure_def)
}

fn expand_storage_decode_impl(
    sc: &Scope<'_>,
    contract: &PPContract<'_>,
    vars: &Vec<(&PPContract<'_>, &PPVariableDef<'_>)>,
) -> syn::Result<TokenStream> {
    if contract.raw.is_interface() || contract.raw.is_library() {
        return Ok(TokenStream::new());
    }

    let sdecode_core = sc.file.sdecode_core();
    let sdecode_solidity = sc.file.sdecode_solidity();
    let sdecode_preimages = sc.file.sdecode_preimages();
    let alloy_primitives = sc.file.alloy_primitives();

    let mut fields_decode = TokenStream::new();
    let mut struct_creation = TokenStream::new();

    for (contract, var) in vars {
        if let Some(slot) = &var.attrs.slot {
            fields_decode.extend(quote! {
                let remaining = #sdecode_core::StorageReader::consume_remaining(&mut storage_reader);
                if remaining.is_not_zero() {
                    return ::core::result::Result::Err(
                        #sdecode_core::StorageError::Layout(
                            #sdecode_solidity::SolLayoutError::remaining_bytes(remaining)
                        )
                    );
                }
                ::core::mem::drop(storage_reader);
                let mut storage_reader = layout.reader_at(#alloy_primitives ::b256!(#slot));
            });
        }

        let field_name = var.raw.name.0.clone().with_span(Span::call_site());

        let field_ty = if let Some(overriden_type) = &var.attrs.typ {
            quote! { #overriden_type }
        } else {
            get_default_rust_type(&sc.in_contract(contract), &var.raw.ty)?
        };

        let field_sol_ty = get_sol_storage_type(&sc.in_contract(contract), &var.raw.ty)?;

        fields_decode.extend(quote! {
            let #field_name = <
                #field_ty
                as ::sdecode_solidity ::SolStorageValue<#field_sol_ty>
            >::decode_storage(&mut storage_reader)
                .map_err(#sdecode_core ::StorageError::Layout)?;
        });

        struct_creation.extend(quote! { #field_name, });
    }

    let language = if let Some(language) = &contract.attrs.language {
        match language.value().to_lowercase().as_ref() {
            "solidity" => quote! { SOLIDITY },
            "vyper" => quote! { VYPER },
            _ => return Err(syn::Error::new_spanned(language, "unknown language")),
        }
    } else {
        quote! { SOLIDITY }
    };

    let storage_structure_path = contract.rust_path();

    let storage_decode_impl = quote! {
        #[automatically_derived]
        #[allow(
            non_camel_case_types,
            non_snake_case,
            clippy::pub_underscore_fields,
            clippy::style
        )]
        impl #sdecode_core ::StorageDecode for #storage_structure_path {
            type LayoutError = #sdecode_solidity::SolLayoutError;

            fn sdecode_mut<P, E>(
                preimages_provider: &mut P,
                storage_entries: E,
            ) -> ::core::result::Result<Self, #sdecode_core ::StorageError<P::Error, Self::LayoutError>>
            where
                P: #sdecode_preimages ::PreimagesProviderMut,
                E: ::core::iter::IntoIterator<Item = (#alloy_primitives::B256, #alloy_primitives::B256)>,
            {
                let side = #sdecode_core::MappingKeySide:: #language;

                let mut layout = #sdecode_core ::Storage::decode_mut(preimages_provider, storage_entries, side)
                    .map_err(#sdecode_core ::StorageError::Provider)?;
                let mut storage_reader = layout.reader_at(#alloy_primitives::B256::ZERO);

                #fields_decode

                ::core::mem::drop(storage_reader);

                ::core::result::Result::Ok(Self { #struct_creation })
            }
        }
    };

    Ok(storage_decode_impl)
}
