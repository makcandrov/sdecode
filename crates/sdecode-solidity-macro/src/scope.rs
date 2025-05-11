use proc_macro2::TokenStream;
use quick_impl::quick_impl;
use quote::{ToTokens, quote};
use syn::Ident;
use syn_solidity::{SolIdent, SolPath};

use crate::pp::{PPContract, PPFile, UserDefinedItem};

#[derive(Debug, Clone)]
pub struct Scope<'a> {
    pub file: &'a PPFile<'a>,
    pub contract: Option<&'a PPContract<'a>>,
}

#[derive(Debug, Clone)]
#[quick_impl]
pub struct Scoped<'a, T> {
    #[quick_impl(impl Deref, impl DerefMut)]
    pub inner: T,
    pub scope: Scope<'a>,
}

impl<'a> Scope<'a> {
    pub const fn top_level(file: &'a PPFile<'a>) -> Self {
        Self {
            file,
            contract: None,
        }
    }

    pub const fn in_contract(&self, contract: &'a PPContract<'a>) -> Self {
        Self {
            file: self.file,
            contract: Some(contract),
        }
    }

    pub const fn top_level_scopped<T>(&self, inner: T) -> Scoped<'a, T> {
        Scoped {
            inner,
            scope: Self::top_level(self.file),
        }
    }

    pub const fn in_contract_scopped<T>(
        &self,
        inner: T,
        contract: &'a PPContract<'a>,
    ) -> Scoped<'a, T> {
        Scoped {
            inner,
            scope: self.in_contract(contract),
        }
    }

    pub fn super_kw(&self) -> TokenStream {
        if self.contract.is_some() {
            quote! { super:: }
        } else {
            TokenStream::new()
        }
    }

    pub fn with_super_kw(&self, tokens: impl ToTokens) -> TokenStream {
        let super_kw = self.super_kw();
        quote! { #super_kw #tokens}
    }
}

impl<'a> Scope<'a> {
    pub fn user_defined_item_ident(
        &self,
        ident: &Ident,
    ) -> Option<Scoped<'a, &'a UserDefinedItem<'a>>> {
        if let Some(contract) = &self.contract {
            if let Some(udi) = contract.udis.get(ident) {
                return Some(self.in_contract_scopped(udi, contract));
            }
        }
        self.file
            .udis
            .get(ident)
            .map(|udi| self.top_level_scopped(udi))
    }

    pub fn user_defined_item_path<'b, 'c>(
        &'b self,
        path: &'c SolPath,
    ) -> Result<Scoped<'a, &'a UserDefinedItem<'a>>, &'c SolIdent> {
        let mut path_iter = path.iter();
        let first = path_iter.next().expect("path must not be empty");

        if let Some(contract) = &self.contract {
            if let Some(udi) = contract.udis.get(&first.0) {
                if let Some(next) = path_iter.next() {
                    return Err(next);
                } else {
                    return Ok(self.in_contract_scopped(udi, contract));
                }
            }
        }

        let udi = self.file.udis.get(&first.0).ok_or(first)?;

        if let Some(next) = path_iter.next() {
            if let Some(contract) = udi.as_contract() {
                let udi = contract.udis.get(&next.0).ok_or(next)?;
                Ok(self.in_contract_scopped(udi, contract))
            } else {
                Err(next)
            }
        } else {
            Ok(self.top_level_scopped(udi))
        }
    }
}
