use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Ident, Lit, LitStr, Path, meta::ParseNestedMeta, parse::Parse};

pub const SDECODE_MACRO_HELPER: &str = "sdecode";

pub const TYPE_ATTR: &str = "type";
pub const RENAME_ATTR: &str = "rename";
pub const REMOTE_ATTR: &str = "remote";
pub const SLOT_ATTR: &str = "slot";
pub const LANGUAGE_ATTR: &str = "language";
pub const REEXPORT_ATTR: &str = "reexport";

#[derive(Debug, Clone, Default)]
pub struct StorageVariableAttrs {
    pub typ: Option<Path>,
    pub slot: Option<LitStr>,
}

#[derive(Debug, Clone, Default)]
pub struct StructureAttrs {
    pub rename: Option<Ident>,
    pub remote: Option<Path>,
}

#[derive(Debug, Clone, Default)]
pub struct StructureFieldAttrs {
    pub typ: Option<Path>,
}

#[derive(Debug, Clone, Default)]
pub struct UdtAttrs {
    pub typ: Option<Path>,
    pub rename: Option<Ident>,
    pub remote: Option<Path>,
}

#[derive(Debug, Clone, Default)]
pub struct ContractAttrs {
    pub rename: Option<Ident>,
    pub remote: Option<Path>,
    pub language: Option<LitStr>,
}

#[derive(Debug, Clone, Default)]
pub struct GlobalAttrs {
    pub reexport: Option<Path>,
}

impl StorageVariableAttrs {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<(Self, TokenStream)> {
        let mut res = Self::default();
        let mut remaining = Vec::new();
        for attr in attrs {
            if !attr.path().is_ident(SDECODE_MACRO_HELPER) {
                remaining.push(attr);
                continue;
            }

            attr.meta.require_list()?.parse_nested_meta(|meta| {
                if meta.path.is_ident(TYPE_ATTR) {
                    let path = Path::parse(meta.value()?)?;
                    let old = res.typ.replace(path);
                    return check_duplicate(old, &meta);
                }

                if meta.path.is_ident(SLOT_ATTR) {
                    let Lit::Str(lit_str) = Lit::parse(meta.value()?)? else {
                        return Err(meta.error("expected string literal"));
                    };

                    let old = res.slot.replace(lit_str);
                    return check_duplicate(old, &meta);
                }

                Err(meta.error("unrecognized attribute"))
            })?;
        }
        Ok((res, attrs_tokens(remaining)))
    }
}

impl StructureAttrs {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<(Self, TokenStream)> {
        let mut res = Self::default();
        let mut remaining = Vec::new();
        for attr in attrs {
            if !attr.path().is_ident(SDECODE_MACRO_HELPER) {
                remaining.push(attr);
                continue;
            }

            attr.meta.require_list()?.parse_nested_meta(|meta| {
                if meta.path.is_ident(RENAME_ATTR) {
                    let ident = Ident::parse(meta.value()?)?;
                    let old = res.rename.replace(ident);
                    return check_duplicate(old, &meta);
                }

                if meta.path.is_ident(REMOTE_ATTR) {
                    let path = Path::parse(meta.value()?)?;
                    let old = res.remote.replace(path);
                    return check_duplicate(old, &meta);
                }

                Err(meta.error("unrecognized attribute"))
            })?;
        }
        Ok((res, attrs_tokens(remaining)))
    }
}

impl StructureFieldAttrs {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<(Self, TokenStream)> {
        let mut res = Self::default();
        let mut remaining = Vec::new();
        for attr in attrs {
            if !attr.path().is_ident(SDECODE_MACRO_HELPER) {
                remaining.push(attr);
                continue;
            }

            attr.meta.require_list()?.parse_nested_meta(|meta| {
                if meta.path.is_ident(TYPE_ATTR) {
                    let path = Path::parse(meta.value()?)?;
                    let old = res.typ.replace(path);
                    return check_duplicate(old, &meta);
                }

                Err(meta.error("unrecognized attribute"))
            })?;
        }
        Ok((res, attrs_tokens(remaining)))
    }
}

impl UdtAttrs {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<(Self, TokenStream)> {
        let mut res = Self::default();
        let mut remaining = Vec::new();
        for attr in attrs {
            if !attr.path().is_ident(SDECODE_MACRO_HELPER) {
                remaining.push(attr);
                continue;
            }

            attr.meta.require_list()?.parse_nested_meta(|meta| {
                if meta.path.is_ident(TYPE_ATTR) {
                    let path = Path::parse(meta.value()?)?;
                    let old = res.typ.replace(path);
                    check_duplicate(old, &meta)
                } else if meta.path.is_ident(RENAME_ATTR) {
                    let ident = Ident::parse(meta.value()?)?;
                    let old = res.rename.replace(ident);
                    check_duplicate(old, &meta)
                } else if meta.path.is_ident(REMOTE_ATTR) {
                    let path = Path::parse(meta.value()?)?;
                    let old = res.remote.replace(path);
                    check_duplicate(old, &meta)
                } else {
                    Err(meta.error("unrecognized attribute"))
                }
            })?;
        }
        Ok((res, attrs_tokens(remaining)))
    }
}

impl ContractAttrs {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<(Self, TokenStream)> {
        let mut res = Self::default();
        let mut remaining = Vec::new();
        for attr in attrs {
            if !attr.path().is_ident(SDECODE_MACRO_HELPER) {
                remaining.push(attr);
                continue;
            }

            attr.meta.require_list()?.parse_nested_meta(|meta| {
                if meta.path.is_ident(RENAME_ATTR) {
                    let ident = Ident::parse(meta.value()?)?;
                    let old = res.rename.replace(ident);
                    check_duplicate(old, &meta)
                } else if meta.path.is_ident(REMOTE_ATTR) {
                    let path = Path::parse(meta.value()?)?;
                    let old = res.remote.replace(path);
                    check_duplicate(old, &meta)
                } else if meta.path.is_ident(LANGUAGE_ATTR) {
                    let Lit::Str(lit_str) = Lit::parse(meta.value()?)? else {
                        return Err(meta.error("expected string literal"));
                    };

                    let old = res.language.replace(lit_str);
                    check_duplicate(old, &meta)
                } else {
                    Err(meta.error("unrecognized attribute"))
                }
            })?;
        }
        Ok((res, attrs_tokens(remaining)))
    }
}

impl GlobalAttrs {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<(Self, TokenStream)> {
        let mut res = Self::default();
        let mut remaining = Vec::new();
        for attr in attrs {
            if !attr.path().is_ident(SDECODE_MACRO_HELPER) {
                remaining.push(attr);
                continue;
            }

            attr.meta.require_list()?.parse_nested_meta(|meta| {
                if meta.path.is_ident(REEXPORT_ATTR) {
                    let path = Path::parse(meta.value()?)?;
                    let old = res.reexport.replace(path);
                    check_duplicate(old, &meta)
                } else {
                    Err(meta.error("unrecognized attribute"))
                }
            })?;
        }
        Ok((res, attrs_tokens(remaining)))
    }
}

fn check_duplicate<T>(old: Option<T>, meta: &ParseNestedMeta<'_>) -> syn::Result<()> {
    old.map_or(Ok(()), |_| Err(meta.error("duplicate attribute")))
}

pub fn attrs_tokens<'a>(attrs: impl IntoIterator<Item = &'a Attribute>) -> TokenStream {
    let mut res = TokenStream::new();
    for attr in attrs {
        res.extend(quote! { #attr });
    }
    res
}
