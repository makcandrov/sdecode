use std::{collections::BTreeMap, rc::Rc};

use indexmap::IndexMap;
use proc_macro2::{Span, TokenStream};
use quick_impl::quick_impl_all;
use quote::{ToTokens, format_ident, quote};
use syn::Ident;
use syn_solidity::{
    ContractKind, File, Item, ItemContract, ItemEnum, ItemStruct, ItemUdt, SolIdent, Spanned,
    VariableDefinition,
};

use crate::{
    attribute::{ContractAttrs, GlobalAttrs, StorageVariableAttrs, StructureAttrs, UdtAttrs},
    linearize::c3_linearize,
    utils::to_snake_case,
};

#[derive(Debug, Clone)]
pub struct PPFile<'a> {
    pub attrs: GlobalAttrs,
    pub remaining_attrs: TokenStream,

    /// Ident => user defined item
    pub udis: IndexMap<Ident, UserDefinedItem<'a>>,

    pub contracts: Vec<Rc<PPContract<'a>>>,

    #[expect(unused)]
    pub raw: &'a File,
}

impl<'a> PPFile<'a> {
    pub fn pre_process(file: &'a File) -> syn::Result<Self> {
        let (attrs, remaining_attrs) = GlobalAttrs::parse(&file.attrs)?;

        let mut contract_indexes = BTreeMap::<SolIdent, (usize, &ItemContract)>::new();
        for contract in iter_contracts(&file.items) {
            contract_indexes.insert(contract.name.clone(), (contract_indexes.len(), contract));
        }

        let mut inheritance_list = Vec::new();
        for contract in iter_contracts(&file.items) {
            let Some(inheritance) = &contract.inheritance else {
                inheritance_list.push(Vec::new());
                continue;
            };

            let mut parents = Vec::new();
            for modifier in inheritance.inheritance.iter().rev() {
                if modifier.name.len() != 1 {
                    return Err(syn::Error::new(
                        modifier.name.span(),
                        "invalid inherited contract name",
                    ));
                }
                let parent_name = modifier.name.first();
                let Some((parent_index, parent_contract)) =
                    contract_indexes.get(parent_name).copied()
                else {
                    let msg = format!(
                        "Could not find inherited contract. If this contract doesn't have any storage variable, you can either remove this inheritance, or add a blank interface at the root of the file: `interface {} {{}}`",
                        parent_name,
                    );
                    return Err(syn::Error::new(parent_name.span(), msg));
                };

                match (contract.kind, parent_contract.kind) {
                    (ContractKind::Library(..), _) => {
                        return Err(syn::Error::new(
                            parent_name.span(),
                            "library is not allowed to inherit",
                        ));
                    }
                    (_, ContractKind::Library(..)) => {
                        return Err(syn::Error::new(
                            parent_name.span(),
                            "libraries cannot be inherited from",
                        ));
                    }
                    (
                        ContractKind::Interface(..),
                        ContractKind::AbstractContract(..) | ContractKind::Contract(..),
                    ) => {
                        return Err(syn::Error::new(
                            parent_name.span(),
                            "interfaces can only inherit from other interfaces",
                        ));
                    }
                    _ => {}
                }
                parents.push(parent_index);
            }
            inheritance_list.push(parents);
        }

        let linearized = match c3_linearize(&inheritance_list) {
            Ok(linearized) => linearized,
            Err(contract_id) => {
                let msg = "linearization of inheritance graph impossible; maybe due to the presence of cycles";
                let contract_ident = contract_indexes
                    .iter()
                    .find(|(_, (id, _))| *id == contract_id)
                    .unwrap()
                    .0;
                return Err(syn::Error::new(contract_ident.span(), msg));
            }
        };

        let mut linearized_iter = linearized.into_iter();

        let mut udis = IndexMap::new();
        let mut contracts = Vec::new();

        for item in &file.items {
            let (ident, udi) = match item {
                Item::Contract(contract) => {
                    let pp_contract = Rc::new(PPContract::pre_process(
                        contract,
                        linearized_iter.next().unwrap(),
                    )?);
                    contracts.push(pp_contract.clone());
                    (&contract.name, UserDefinedItem::Contract(pp_contract))
                }
                Item::Enum(enumm) => (
                    &enumm.name,
                    UserDefinedItem::Enum(PPEnum::pre_process(enumm)?),
                ),
                Item::Struct(structure) => (
                    &structure.name,
                    UserDefinedItem::Struct(PPStruct::pre_process(structure)?),
                ),
                Item::Udt(udt) => (&udt.name, UserDefinedItem::Udt(PPUdt::pre_process(udt)?)),
                Item::Event(event) => (&event.name, UserDefinedItem::Event),
                Item::Error(error) => (&error.name, UserDefinedItem::Error),
                _ => continue,
            };

            let old = udis.insert(ident.0.clone(), udi);
            if old.is_some() {
                return Err(syn::Error::new(ident.span(), "identifier already declared"));
            }
        }

        Ok(Self {
            attrs,
            remaining_attrs,
            udis,
            contracts,
            // structs,
            // enums,
            // udts,
            raw: file,
        })
    }

    pub fn sdecode_solidity(&self) -> TokenStream {
        if let Some(reexport) = &self.attrs.reexport {
            reexport.to_token_stream()
        } else {
            quote! { ::sdecode_solidity }
        }
    }

    pub fn sdecode_solidity_data_types(&self) -> TokenStream {
        let sdecode_solidity = &self.sdecode_solidity();
        quote! { #sdecode_solidity :: data_types }
    }

    pub fn sdecode_core(&self) -> TokenStream {
        let sdecode_solidity = &self.sdecode_solidity();
        quote! { #sdecode_solidity :: __private::sdecode_core }
    }

    pub fn sdecode_preimages(&self) -> TokenStream {
        let sdecode_solidity = &self.sdecode_solidity();
        quote! { #sdecode_solidity :: __private::sdecode_preimages }
    }

    pub fn alloy_primitives(&self) -> TokenStream {
        let sdecode_solidity = &self.sdecode_solidity();
        quote! { #sdecode_solidity :: __private::alloy_primitives }
    }

    pub fn parents_of(&self, contract: &PPContract<'a>) -> impl Iterator<Item = &PPContract<'a>> {
        contract
            .parent_contracts
            .iter()
            .rev()
            .map(|index| self.contracts[*index].as_ref())
    }
}

fn iter_contracts(items: &[Item]) -> impl Iterator<Item = &ItemContract> {
    items.iter().filter_map(|item| match item {
        Item::Contract(contract) => Some(contract),
        _ => None,
    })
}

#[derive(Debug, Clone)]
pub struct PPContract<'a> {
    pub attrs: ContractAttrs,
    pub remaining_attrs: TokenStream,

    /// C3-linearized inheritance list
    pub parent_contracts: Vec<usize>,

    pub udis: IndexMap<Ident, UserDefinedItem<'a>>,

    // pub structs: Vec<PPStruct<'a>>,
    // pub enums: Vec<PPEnum<'a>>,
    // pub udts: Vec<PPUdt<'a>>,
    // pub storage_vars: Vec<PPVariableDef<'a>>,
    pub raw: &'a ItemContract,
}

impl<'a> PPContract<'a> {
    pub fn pre_process(
        contract: &'a ItemContract,
        parent_contracts: Vec<usize>,
    ) -> syn::Result<Self> {
        let (attrs, remaining_attrs) = ContractAttrs::parse(&contract.attrs)?;

        let mut udis = IndexMap::new();

        for item in &contract.body {
            let (ident, udi) = match item {
                Item::Contract(contract) => {
                    return Err(syn::Error::new(
                        contract.span(),
                        "nested contracts are disallowed",
                    ));
                }
                Item::Enum(enumm) => (
                    &enumm.name,
                    UserDefinedItem::Enum(PPEnum::pre_process(enumm)?),
                ),
                Item::Struct(structure) => (
                    &structure.name,
                    UserDefinedItem::Struct(PPStruct::pre_process(structure)?),
                ),
                Item::Variable(variable) => (
                    &variable.name,
                    UserDefinedItem::Variable(PPVariableDef::pre_process(variable)?),
                ),
                Item::Udt(udt) => (&udt.name, UserDefinedItem::Udt(PPUdt::pre_process(udt)?)),
                Item::Event(event) => (&event.name, UserDefinedItem::Event),
                Item::Error(error) => (&error.name, UserDefinedItem::Error),
                _ => continue,
            };

            let old = udis.insert(ident.0.clone(), udi);
            if old.is_some() {
                return Err(syn::Error::new(ident.span(), "identifier already declared"));
            }
        }

        Ok(Self {
            attrs,
            remaining_attrs,
            parent_contracts,
            udis,
            // structs,
            // enums,
            // udts,
            // storage_vars,
            raw: contract,
        })
    }

    pub fn rust_path(&self) -> TokenStream {
        if let Some(remote) = &self.attrs.remote {
            remote.to_token_stream()
        } else if let Some(rename) = &self.attrs.rename {
            rename.to_token_stream()
        } else {
            format_ident!("{}Storage", self.raw.name, span = self.raw.name.span()).to_token_stream()
        }
    }

    pub fn mod_name(&self) -> Ident {
        let ident = if let Some(remote) = &self.attrs.remote {
            &remote.segments.last().unwrap().ident
        } else if let Some(rename) = &self.attrs.rename {
            rename
        } else {
            &self.raw.name.0
        };
        let snakecase = to_snake_case(&ident.to_string());
        Ident::new(&snakecase, Span::call_site())
    }
}

#[derive(Debug, Clone)]
pub struct PPStruct<'a> {
    pub attrs: StructureAttrs,
    pub remaining_attrs: TokenStream,
    pub raw: &'a ItemStruct,
}

impl<'a> PPStruct<'a> {
    pub fn pre_process(structure: &'a ItemStruct) -> syn::Result<Self> {
        let (attrs, remaining_attrs) = StructureAttrs::parse(&structure.attrs)?;

        Ok(Self {
            attrs,
            remaining_attrs,
            raw: structure,
        })
    }

    pub fn rust_path(&self) -> TokenStream {
        if let Some(remote) = &self.attrs.remote {
            remote.to_token_stream()
        } else if let Some(rename) = &self.attrs.rename {
            rename.to_token_stream()
        } else {
            self.raw.name.to_token_stream()
        }
    }

    pub fn rust_ident(&self) -> &Ident {
        if let Some(remote) = &self.attrs.remote {
            &remote.segments.last().unwrap().ident
        } else if let Some(rename) = &self.attrs.rename {
            rename
        } else {
            &self.raw.name.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct PPEnum<'a> {
    pub attrs: StructureAttrs,
    pub remaining_attrs: TokenStream,
    pub raw: &'a ItemEnum,
}

impl<'a> PPEnum<'a> {
    pub fn pre_process(enumm: &'a ItemEnum) -> syn::Result<Self> {
        let (attrs, remaining_attrs) = StructureAttrs::parse(&enumm.attrs)?;

        Ok(Self {
            attrs,
            remaining_attrs,
            raw: enumm,
        })
    }

    pub fn rust_path(&self) -> TokenStream {
        if let Some(renmote) = &self.attrs.remote {
            renmote.to_token_stream()
        } else if let Some(rename) = &self.attrs.rename {
            rename.to_token_stream()
        } else {
            self.raw.name.to_token_stream()
        }
    }

    pub fn rust_ident(&self) -> &Ident {
        if let Some(remote) = &self.attrs.remote {
            &remote.segments.last().unwrap().ident
        } else if let Some(rename) = &self.attrs.rename {
            rename
        } else {
            &self.raw.name.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct PPUdt<'a> {
    pub attrs: UdtAttrs,
    pub remaining_attrs: TokenStream,
    pub raw: &'a ItemUdt,
}

impl<'a> PPUdt<'a> {
    pub fn pre_process(udt: &'a ItemUdt) -> syn::Result<Self> {
        let (attrs, remaining_attrs) = UdtAttrs::parse(&udt.attrs)?;

        Ok(Self {
            attrs,
            remaining_attrs,
            raw: udt,
        })
    }

    pub fn rust_path(&self) -> TokenStream {
        if let Some(renmote) = &self.attrs.remote {
            renmote.to_token_stream()
        } else if let Some(rename) = &self.attrs.rename {
            rename.to_token_stream()
        } else {
            self.raw.name.to_token_stream()
        }
    }

    pub fn rust_ident(&self) -> &Ident {
        if let Some(remote) = &self.attrs.remote {
            &remote.segments.last().unwrap().ident
        } else if let Some(rename) = &self.attrs.rename {
            rename
        } else {
            &self.raw.name.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct PPVariableDef<'a> {
    pub attrs: StorageVariableAttrs,
    pub remaining_attrs: TokenStream,
    pub raw: &'a VariableDefinition,
}

impl<'a> PPVariableDef<'a> {
    pub fn pre_process(var: &'a VariableDefinition) -> syn::Result<Self> {
        let (attrs, remaining_attrs) = StorageVariableAttrs::parse(&var.attrs)?;

        Ok(Self {
            attrs,
            remaining_attrs,
            raw: var,
        })
    }
}

/// Any use defined item *inside of a contract* that is considered by the compiler as potentially a
/// type.
///
/// Note that this enum excludes functions, as they are ignored by the compiler when resolving
/// types. For example:
///
/// ```solidity
/// struct foo {
///     uint256 bar;
/// }
///
/// contract Contract {
///     // even though `foo` shadows the structure definition ...
///     function foo() external;
///
///     struct Bar {
///         /// ... the type `foo` is considered referencing the structure and not the function
///         foo bar;
///     }
/// }
/// ```
#[derive(Debug, Clone)]
#[quick_impl_all(pub const is, pub as_ref)]
pub enum UserDefinedItem<'a> {
    // Pre processed
    Contract(Rc<PPContract<'a>>),
    Struct(PPStruct<'a>),
    Enum(PPEnum<'a>),
    Udt(PPUdt<'a>),
    Variable(PPVariableDef<'a>),

    // Raw
    Error,
    Event,
}
