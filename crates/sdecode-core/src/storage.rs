use std::collections::{BTreeMap, btree_map};

use alloy_primitives::{B256, Bytes, U256};
use overf::checked;
use sdecode_preimages::{PreimagesProvider, PreimagesProviderMut, caches::StoragePreimagesCache};

use crate::{
    AnchorKind, MAX_STORAGE_OFFSET, MappingKeySide, StorageItem, StorageNode, StorageReader,
    StorageStructure, reader::StorageReaderImpl, utils::b256_to_u256,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Storage {
    pub anchors: BTreeMap<B256, StorageNode>,
    pub undecoded: BTreeMap<B256, (Bytes, StorageStructure)>,
}

impl Storage {
    pub fn decode<P: PreimagesProvider>(
        provider: P,
        storage_entries: impl IntoIterator<Item = (B256, B256)>,
        side: MappingKeySide,
    ) -> Result<Self, P::Error> {
        Self::decode_mut(
            &mut StoragePreimagesCache::new(provider, U256::from(MAX_STORAGE_OFFSET)),
            storage_entries,
            side,
        )
    }

    pub fn decode_mut<P: PreimagesProviderMut>(
        provider: &mut P,
        storage_entries: impl IntoIterator<Item = (B256, B256)>,
        side: MappingKeySide,
    ) -> Result<Self, P::Error> {
        let mut layout = Self::default();

        for (slot, value) in storage_entries {
            let item = StorageItem::decode_mut(provider, side, slot, value)?;

            match item.kind {
                AnchorKind::UnknownPreimage { link } => match layout.anchors.entry(item.anchor) {
                    btree_map::Entry::Vacant(vacant_entry) => {
                        let node = StorageNode::from_link(link);
                        vacant_entry.insert(node);
                    }
                    btree_map::Entry::Occupied(mut occupied_entry) => {
                        occupied_entry.get_mut().add_link(link);
                    }
                },
                AnchorKind::UndecodablePreimage { preimage, chain } => {
                    match layout.undecoded.entry(item.anchor) {
                        btree_map::Entry::Vacant(vacant_entry) => {
                            let structure = StorageStructure::from_chain(chain);
                            vacant_entry.insert((preimage, structure));
                        }
                        btree_map::Entry::Occupied(mut occupied_entry) => {
                            let (current_preimage, structure) = occupied_entry.get_mut();
                            debug_assert_eq!(*current_preimage, preimage);
                            structure.add_chain(chain);
                        }
                    }
                }
            }
        }

        Ok(layout)
    }

    pub fn anchor(&self, slot: B256) -> &StorageNode {
        self.anchors
            .get(&slot)
            .unwrap_or(const { &StorageNode::empty() })
    }

    pub fn reader_at(&mut self, slot: B256) -> impl StorageReader {
        let slot_u256 = b256_to_u256(slot);

        // todo: use MAX_STORAGE_OFFSET
        let upper_u256 = slot_u256
            .checked_add(U256::from(usize::MAX))
            .unwrap_or(U256::MAX);
        let upper = B256::from(upper_u256);
        let upper_u256 = if let Some((upper, _)) = self.anchors.range(slot..=upper).last() {
            b256_to_u256(*upper)
        } else {
            slot_u256
        };

        // Should not panic with the above computations
        let max_delta = usize::try_from(checked! { upper_u256 - slot_u256 }).unwrap();

        StorageReaderImpl::new((0..=max_delta).filter_map(move |i| {
            let i = U256::from(i);
            let slot = B256::from(slot_u256.checked_add(i)?);
            Some(self.anchors.remove(&slot).unwrap_or_default())
        }))
    }
}
