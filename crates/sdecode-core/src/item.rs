use alloy_preimages::{PreimagesProvider, PreimagesProviderMut, WrapPreimagesProvider};
use alloy_primitives::{B256, Bytes};
use quick_impl::quick_impl_all;

use crate::{MappingEntryLocation, MappingKeySide, ResolvedSlot};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct StorageItem {
    pub anchor: B256,
    pub kind: AnchorKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum AnchorKind {
    UnknownPreimage { link: HashLink },
    UndecodablePreimage { preimage: Bytes, chain: HashChain },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct HashChain {
    pub offset: usize,
    pub link: HashLink,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[quick_impl_all(pub const is, pub const from = "{}")]
pub enum HashLink {
    Leaf {
        value: B256,
    },

    Inner {
        key: Bytes,
        remaining_chain: Box<HashChain>,
    },
}

impl StorageItem {
    #[inline]
    pub fn decode<P: PreimagesProvider>(
        provider: P,
        side: MappingKeySide,
        slot: B256,
        value: B256,
    ) -> Result<Self, P::Error> {
        Self::decode_mut(&mut WrapPreimagesProvider(provider), side, slot, value)
    }

    #[inline]
    pub fn decode_mut<P: PreimagesProviderMut>(
        provider: &mut P,
        side: MappingKeySide,
        mut slot: B256,
        value: B256,
    ) -> Result<Self, P::Error> {
        let mut child_link = HashLink::Leaf { value };

        loop {
            let Some(resolved_slot) = ResolvedSlot::decode_mut(provider, slot)? else {
                let item = Self {
                    anchor: slot,
                    kind: AnchorKind::UnknownPreimage { link: child_link },
                };
                return Ok(item);
            };

            let (anchor, offset, preimage) = resolved_slot.into_parts();

            match MappingEntryLocation::try_from_preimage(side, preimage) {
                Ok(MappingEntryLocation {
                    entry_key,
                    mapping_slot,
                }) => {
                    let child = HashLink::Inner {
                        key: entry_key,
                        remaining_chain: Box::new(HashChain {
                            offset,
                            link: child_link,
                        }),
                    };
                    child_link = child;
                    slot = mapping_slot;
                }
                Err(preimage) => {
                    let item = Self {
                        anchor,
                        kind: AnchorKind::UndecodablePreimage {
                            preimage,
                            chain: HashChain {
                                offset,
                                link: child_link,
                            },
                        },
                    };
                    return Ok(item);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy_preimages::providers::InMemoryPreimages;
    use alloy_primitives::{b256, bytes};

    use super::*;

    #[test]
    fn test_layout_item() {
        let mut provider = InMemoryPreimages::new();
        assert_eq!(
            provider.insert(bytes!("0x000000000000000000000000f228183dde65b6a36f5382693636c2ddaadb87a9c7fe799710c4d03c47be714190f54817f9938592f8110a8fc890f12b138782ab")),
            b256!("0x826be66ee1ebb76116a8d7b90e41b55fb5738ec81b19a4ffd22fed7cf95c28f3"),
        );
        assert_eq!(
            provider.insert(bytes!("0x60fce64eeeaec462a3fdf674f786ad71e4eef6e717d848d992a8631a5cb0b4b20000000000000000000000000000000000000000000000000000000000000002")),
            b256!("0xc7fe799710c4d03c47be714190f54817f9938592f8110a8fc890f12b138782ab"),
        );

        let item = StorageItem::decode(
            &provider,
            MappingKeySide::Left,
            b256!("0x826be66ee1ebb76116a8d7b90e41b55fb5738ec81b19a4ffd22fed7cf95c28f7"),
            b256!("0x18ceba3a27c13d260a460c7e1cc13b5d6d954f6df10862f5dc53f49add1617c9"),
        )
        .unwrap();

        assert_eq!(
            item,
            StorageItem {
                anchor: b256!("0x0000000000000000000000000000000000000000000000000000000000000002"),
                kind: AnchorKind::UnknownPreimage {
                    link: HashLink::Inner {
                        key: bytes!(
                            "0x60fce64eeeaec462a3fdf674f786ad71e4eef6e717d848d992a8631a5cb0b4b2"
                        ),
                        remaining_chain: Box::new(HashChain {
                            offset: 0,
                            link: HashLink::Inner {
                                key: bytes!(
                                    "0x000000000000000000000000f228183dde65b6a36f5382693636c2ddaadb87a9"
                                ),
                                remaining_chain: Box::new(HashChain {
                                    offset: 4,
                                    link: HashLink::Leaf {
                                        value: b256!(
                                            "0x18ceba3a27c13d260a460c7e1cc13b5d6d954f6df10862f5dc53f49add1617c9"
                                        ),
                                    },
                                }),
                            },
                        }),
                    },
                },
            }
        );
    }
}
