use std::num::NonZero;

use crate::{array_size::ArraySizeEvaluator, pp::UserDefinedItem, scope::Scope};
use proc_macro2::{Literal, TokenStream};
use quote::{quote, quote_spanned};
use syn::Ident;
use syn_solidity::{Spanned, Type};

pub fn get_sol_storage_type(sc: &Scope<'_>, ty: &Type) -> syn::Result<TokenStream> {
    let data_types = sc.file.sdecode_solidity_data_types();

    let result = match ty {
        Type::Address(span, _payable) => quote_spanned! {*span=> #data_types :: Address },
        Type::Bool(span) => quote_spanned! {*span=> #data_types :: Bool },
        Type::String(span) => quote_spanned! {*span=> #data_types :: String },
        Type::Bytes(span) => quote_spanned! {*span=> #data_types :: Bytes },
        Type::FixedBytes(span, size) => {
            assert!(size.get() <= 32);
            let size = Literal::u16_unsuffixed(size.get());
            quote_spanned! {*span=> #data_types :: FixedBytes<#size> }
        }
        Type::Int(span, size) => {
            let size = size.map_or(256, NonZero::get);
            assert!(size <= 256 && size % 8 == 0);
            let size = Literal::u16_unsuffixed(size);
            quote_spanned! {*span=> #data_types :: Int<#size> }
        }
        Type::Uint(span, size) => {
            let size = size.map_or(256, NonZero::get);
            assert!(size <= 256 && size % 8 == 0);
            let size = Literal::u16_unsuffixed(size);
            quote_spanned! {*span=> #data_types :: Uint<#size> }
        }
        Type::Array(array) => {
            let elem_ty = get_sol_storage_type(sc, &array.ty)?;
            if let Some(size) = &array.size {
                let size = ArraySizeEvaluator::new().eval(sc, size)?;
                let size = Literal::usize_unsuffixed(size);
                quote_spanned! {array.span()=> #data_types :: FixedArray<#elem_ty, #size> }
            } else {
                quote_spanned! {array.span()=> #data_types :: Array<#elem_ty> }
            }
        }
        Type::Tuple(tuple) => {
            return Err(syn::Error::new(
                tuple.span(),
                "tuples are not supported as storage types",
            ));
        }
        Type::Function(function) => quote_spanned! {function.span()=> #data_types :: Function },
        Type::Mapping(mapping) => {
            let key_ty = get_sol_storage_type(sc, &mapping.key)?;
            let value_ty = get_sol_storage_type(sc, &mapping.value)?;
            quote_spanned! {mapping.span()=> #data_types :: Mapping<#key_ty, #value_ty> }
        }
        Type::Custom(path) => match sc.user_defined_item_path(path) {
            Ok(item) => {
                let path_prefix = if let Some(c) = &item.scope.contract {
                    let p = sc.with_super_kw(c.mod_name().with_span(path.first().span()));
                    quote! { #p :: }
                } else {
                    sc.super_kw()
                };
                match &item.inner {
                    UserDefinedItem::Contract(_) => {
                        quote_spanned! {path.span()=> #data_types :: Address }
                    }
                    UserDefinedItem::Struct(structure) => {
                        let ty = structure.rust_path();
                        quote_spanned! {path.span()=> #path_prefix #ty}
                    }
                    UserDefinedItem::Enum(enumm) => {
                        let ty = enumm.rust_path();
                        quote_spanned! {path.span()=> #path_prefix #ty}
                    }
                    UserDefinedItem::Udt(udt) => {
                        let ty = udt.rust_path();
                        quote_spanned! {path.span()=> #path_prefix #ty}
                    }
                    UserDefinedItem::Variable(_) => {
                        return Err(syn::Error::new(
                            path.span(),
                            "variable cannot be resolved as a storage type",
                        ));
                    }
                    UserDefinedItem::Error => {
                        return Err(syn::Error::new(
                            path.span(),
                            "error cannot be resolved as a storage type",
                        ));
                    }
                    UserDefinedItem::Event => {
                        return Err(syn::Error::new(
                            path.span(),
                            "event cannot be resolved as a storage type",
                        ));
                    }
                }
            }
            Err(failed_ident) => {
                return Err(syn::Error::new(failed_ident.span(), "item not found"));
                // if path.len() > 1 {
                //     return Err(syn::Error::new(failed_ident.span(), "item not found"));
                // } else {
                //     let custom_ty = sc.with_super_kw(path.last());
                //     quote_spanned! {path.span()=> #custom_ty }
                // }
            }
        },
    };

    Ok(result)
}

pub fn get_default_rust_type(sc: &Scope<'_>, ty: &Type) -> syn::Result<TokenStream> {
    let alloy_primitives = sc.file.alloy_primitives();

    let result = match ty {
        Type::Address(span, _payable) => quote_spanned! {*span=> #alloy_primitives :: Address },
        Type::Bool(span) => quote_spanned! {*span=> bool },
        Type::String(span) => quote_spanned! {*span=> String },
        Type::Bytes(span) => quote_spanned! {*span=> #alloy_primitives :: Bytes },
        Type::FixedBytes(span, size) => {
            assert!(size.get() <= 32);
            let size = Literal::u16_unsuffixed(size.get());
            quote_spanned! {*span=> #alloy_primitives :: FixedBytes<#size> }
        }
        Type::Int(span, size) => {
            let size = size.map_or(256, NonZero::get);
            assert!(size <= 256 && size % 8 == 0);
            if is_primitive_int(size) {
                let ident = Ident::new(&format!("i{}", size), *span);
                quote_spanned! {*span=> #ident }
            } else {
                let ident = Ident::new(&format!("I{}", size), *span);
                quote_spanned! {*span=> #alloy_primitives ::aliases:: #ident }
            }
        }
        Type::Uint(span, size) => {
            let size = size.map_or(256, NonZero::get);
            assert!(size <= 256 && size % 8 == 0);
            if is_primitive_int(size) {
                let ident = Ident::new(&format!("u{}", size), *span);
                quote_spanned! {*span=> #ident }
            } else {
                let ident = Ident::new(&format!("U{}", size), *span);
                quote_spanned! {*span=> #alloy_primitives ::aliases:: #ident }
            }
        }
        Type::Array(array) => {
            let elem_ty = get_default_rust_type(sc, &array.ty)?;
            if let Some(size) = &array.size {
                let size = ArraySizeEvaluator::new().eval(sc, size)?;

                // Large fixed-size arrays can cause stack overflows when decoding
                if size <= 32 {
                    let size = Literal::usize_unsuffixed(size);
                    quote_spanned! {array.span()=> [#elem_ty; #size] }
                } else {
                    quote_spanned! {array.span()=> Vec<#elem_ty> }
                }
            } else {
                quote_spanned! {array.span()=> Vec<#elem_ty> }
            }
        }
        Type::Tuple(tuple) => {
            return Err(syn::Error::new(
                tuple.span(),
                "tuples are not supported as storage types",
            ));
        }
        Type::Function(function) => {
            quote_spanned! {function.span()=> #alloy_primitives :: Function }
        }
        Type::Mapping(mapping) => {
            let key_ty = get_default_rust_type(sc, &mapping.key)?;
            if matches!(*mapping.value, Type::Bool(_)) {
                quote_spanned! {mapping.span()=> ::std::collections::BTreeSet<#key_ty> }
            } else {
                let value_ty = get_default_rust_type(sc, &mapping.value)?;
                quote_spanned! {mapping.span()=> ::std::collections::BTreeMap<#key_ty, #value_ty> }
            }
        }
        Type::Custom(path) => match sc.user_defined_item_path(path) {
            Ok(item) => {
                let path_prefix = if let Some(c) = &item.scope.contract {
                    let p = sc.with_super_kw(c.mod_name().with_span(path.first().span()));
                    quote! { #p :: }
                } else {
                    sc.super_kw()
                };
                match &item.inner {
                    UserDefinedItem::Contract(_) => {
                        quote_spanned! {path.span()=> #alloy_primitives :: Address }
                    }
                    UserDefinedItem::Struct(structure) => {
                        let ty = structure.rust_path().with_span(path.last().span());
                        quote_spanned! {path.span()=> #path_prefix #ty}
                    }
                    UserDefinedItem::Enum(enumm) => {
                        let ty = enumm.rust_path().with_span(path.last().span());
                        quote_spanned! {path.span()=> #path_prefix #ty}
                    }
                    UserDefinedItem::Udt(udt) => {
                        let ty = udt.rust_path().with_span(path.last().span());
                        quote_spanned! {path.span()=> #path_prefix #ty}
                    }
                    UserDefinedItem::Variable(_) => {
                        return Err(syn::Error::new(
                            path.span(),
                            "variable cannot be resolved as a storage type",
                        ));
                    }

                    UserDefinedItem::Error => {
                        return Err(syn::Error::new(
                            path.span(),
                            "error cannot be resolved as a storage type",
                        ));
                    }
                    UserDefinedItem::Event => {
                        return Err(syn::Error::new(
                            path.span(),
                            "event cannot be resolved as a storage type",
                        ));
                    }
                }
            }
            Err(failed_ident) => {
                if path.len() > 1 {
                    return Err(syn::Error::new(failed_ident.span(), "item not found"));
                } else {
                    let custom_ty = path.last();
                    quote_spanned! {path.span()=> #custom_ty }
                }
            }
        },
    };

    Ok(result)
}

const fn is_primitive_int(size: u16) -> bool {
    matches!(size, 8 | 16 | 32 | 64 | 128)
}

pub fn is_mapping_key_type(_sc: &Scope<'_>, ty: &Type) -> syn::Result<bool> {
    let res = match ty {
        Type::Uint(..)
        | Type::Int(..)
        | Type::Bytes(_)
        | Type::String(_)
        | Type::FixedBytes(..)
        | Type::Bool(_)
        | Type::Address(..) => true,
        Type::Array(_) | Type::Tuple(_) | Type::Function(_) | Type::Mapping(_) => false,
        Type::Custom(path) => {
            return Err(syn::Error::new(
                path.span(),
                "the underlying type for a user defined value type has to be an elementary value type",
            ));
        }
    };
    Ok(res)
}
