use proc_macro2::TokenStream;

use crate::{
    pp::{PPFile, UserDefinedItem},
    scope::Scope,
};

use super::{ContractExpansion, EnumExpansion, StructureExpansion, UdtExpansion};

#[derive(Debug, Clone, Default)]
pub struct FileExpansion {
    pub global_attributes: TokenStream,
    pub structures: Vec<StructureExpansion>,
    pub udts: Vec<UdtExpansion>,
    pub enums: Vec<EnumExpansion>,
    pub contracts: Vec<ContractExpansion>,
}

impl FileExpansion {
    pub fn expand<'b>(sc: &Scope<'b>, file: &'b PPFile<'_>) -> syn::Result<Self> {
        let mut structures = Vec::new();
        let mut udts = Vec::new();
        let mut enums = Vec::new();
        let mut contracts = Vec::new();

        for udi in file.udis.values() {
            match udi {
                UserDefinedItem::Contract(contract) => {
                    let expansion = ContractExpansion::expand(&sc.in_contract(contract), contract)?;
                    contracts.push(expansion);
                }
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
                _ => {}
            }
        }

        Ok(Self {
            global_attributes: file.remaining_attrs.clone(),
            structures,
            udts,
            enums,
            contracts,
        })
    }

    pub fn into_tokens(self) -> TokenStream {
        let mut res = self.global_attributes;
        for structure in self.structures {
            res.extend(structure.into_tokens());
        }
        for udt in self.udts {
            res.extend(udt.into_tokens());
        }
        for enumm in self.enums {
            res.extend(enumm.into_tokens());
        }
        for contract in self.contracts {
            res.extend(contract.into_tokens());
        }
        res
    }
}
