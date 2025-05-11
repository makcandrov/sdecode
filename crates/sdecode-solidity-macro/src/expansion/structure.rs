use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::LitStr;
use syn_solidity::Spanned;

use crate::{
    attribute::StructureFieldAttrs, get_sol_storage_type, pp::PPStruct, scope::Scope,
    types::get_default_rust_type,
};

#[derive(Debug, Clone, Default)]
pub struct StructureExpansion {
    pub structure_def: TokenStream,
    pub sol_storage_type_impl: TokenStream,
    pub sol_storage_value_impl: TokenStream,
}

impl StructureExpansion {
    pub fn expand(sc: &Scope<'_>, structure: &PPStruct<'_>) -> syn::Result<Self> {
        Ok(Self {
            structure_def: expand_structure_def(sc, structure)?,
            sol_storage_type_impl: expand_sol_storage_type_impl(sc, structure)?,
            sol_storage_value_impl: expand_sol_storage_value_impl(sc, structure)?,
        })
    }

    pub fn into_tokens(self) -> TokenStream {
        let mut res = TokenStream::new();
        res.extend(self.structure_def);
        res.extend(self.sol_storage_type_impl);
        res.extend(self.sol_storage_value_impl);
        res
    }
}

fn expand_structure_def(sc: &Scope<'_>, structure: &PPStruct<'_>) -> syn::Result<TokenStream> {
    if structure.attrs.remote.is_some() {
        return Ok(TokenStream::new());
    }

    let structure_ident = structure.rust_ident();
    let remaining_attrs = &structure.remaining_attrs;

    let mut structure_fields = TokenStream::new();

    for field in &structure.raw.fields {
        let (attrs, remaining_attrs) = StructureFieldAttrs::parse(&field.attrs)?;

        let Some(field_name) = &field.name else {
            return Err(syn::Error::new(
                field.span(),
                "unnamed fields are not supported",
            ));
        };

        let field_ty = if let Some(overriden_type) = attrs.typ {
            quote! { #overriden_type }
        } else {
            get_default_rust_type(sc, &field.ty)?
        };

        structure_fields.extend(quote! {
            #[allow(missing_docs)]
            #remaining_attrs
            pub #field_name: #field_ty,
        });
    }

    let structure_def = quote! {
        #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
        #remaining_attrs
        pub struct #structure_ident {
            #structure_fields
        }
    };

    Ok(structure_def)
}

fn expand_sol_storage_type_impl(
    sc: &Scope<'_>,
    structure: &PPStruct<'_>,
) -> syn::Result<TokenStream> {
    let structure_path = structure.rust_path();
    let structure_ident = structure.rust_ident();
    let structure_ident_lit = LitStr::new(&structure_ident.to_string(), structure_ident.span());

    let sdecode_solidity = sc.file.sdecode_solidity();

    let sol_storage_type_impl = quote! {
        #[automatically_derived]
        #[allow(
            non_camel_case_types,
            non_snake_case,
            clippy::pub_underscore_fields,
            clippy::style
        )]
        impl #sdecode_solidity::SolStorageType for #structure_path {
            const SOL_STORAGE_NAME: &str = #structure_ident_lit;
        }
    };

    Ok(sol_storage_type_impl)
}

fn expand_sol_storage_value_impl(
    sc: &Scope<'_>,
    structure: &PPStruct<'_>,
) -> syn::Result<TokenStream> {
    let structure_path = structure.rust_path();

    let sdecode_solidity = sc.file.sdecode_solidity();
    let sdecode_core = sc.file.sdecode_core();

    let mut destructure_fields = TokenStream::new();
    let mut sol_storage_types = TokenStream::new();

    for field in &structure.raw.fields {
        let Some(field_name) = &field.name else {
            return Err(syn::Error::new(
                field.span(),
                "unnamed fields are not supported",
            ));
        };
        let field_name_unspanned = field_name.0.clone().with_span(Span::call_site());

        destructure_fields.extend(quote! { #field_name_unspanned, });

        let sol_storage_typ = get_sol_storage_type(sc, &field.ty)?;
        sol_storage_types.extend(quote! { #sol_storage_typ, });
    }
    sol_storage_types = quote! { (#sol_storage_types) };

    let sol_storage_value_impl = quote! {
        #[automatically_derived]
        #[allow(
            non_camel_case_types,
            non_snake_case,
            clippy::pub_underscore_fields,
            clippy::style
        )]
        impl #sdecode_solidity::SolStorageValue<Self> for #structure_path {
            fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, #sdecode_solidity::SolLayoutError>
            where
                Reader: #sdecode_core::StorageReader,
            {
                let (#destructure_fields) = <
                    #sdecode_solidity::helpers::SolStructureHelper<Self, _, #sol_storage_types>
                    as #sdecode_solidity::SolStorageValue<_>
                >::decode_storage(storage_reader)?.0;

                Ok(Self { #destructure_fields })
            }
        }
    };

    Ok(sol_storage_value_impl)
}
