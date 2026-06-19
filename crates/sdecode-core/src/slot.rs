use alloy_preimages::{
    PreimageEntry, PreimageEntryRef, PreimagesProvider, PreimagesProviderMut, WrapPreimagesProvider,
};
use alloy_primitives::{B256, Bytes, U256};
use quick_impl::quick_impl;

use crate::utils::b256_to_u256;

pub const MAX_STORAGE_OFFSET: usize = 0xffffffffffff;
pub const MAX_STORAGE_OFFSET_U256: U256 = U256::from_be_slice(&MAX_STORAGE_OFFSET.to_be_bytes());

/// A storage slot resolved to the nearest known preimage at or below it.
///
/// The queried slot equals `keccak(preimage) + offset`: it sits `offset` words
/// above `anchor`, where `anchor == keccak(preimage)`.
///
/// The fields are mutually dependent, so the only way to obtain a
/// [`ResolvedSlot`] is through [`ResolvedSlot::decode`] (or [`decode_mut`]),
/// which guarantees they are consistent and that `offset <= MAX_STORAGE_OFFSET`.
///
/// [`decode_mut`]: ResolvedSlot::decode_mut
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[quick_impl(pub const new = "new_unchecked", pub into_parts)]
pub struct ResolvedSlot {
    /// The base slot with a known preimage (`keccak(preimage)`).
    #[quick_impl(pub get_clone = "{}")]
    anchor: B256,

    /// Word offset of the queried slot relative to `anchor`.
    #[quick_impl(pub get_clone = "{}")]
    offset: usize,

    /// The preimage that hashes to `anchor`.
    #[quick_impl(pub get = "{}", into)]
    preimage: Bytes,
}

impl ResolvedSlot {
    pub fn decode<P: PreimagesProvider>(provider: P, slot: B256) -> Result<Option<Self>, P::Error> {
        Self::decode_mut(&mut WrapPreimagesProvider(provider), slot)
    }

    pub fn decode_mut<P: PreimagesProviderMut>(
        provider: &mut P,
        slot: B256,
    ) -> Result<Option<Self>, P::Error> {
        let Some(entry) = provider.nearest_lower_preimage_mut(&slot)? else {
            return Ok(None);
        };

        let (anchor, preimage) = entry.into_parts();

        let offset = b256_to_u256(slot)
            .checked_sub(b256_to_u256(anchor))
            .expect("should be lower");

        if let Some(offset) = as_offset(offset) {
            Ok(Some(Self {
                anchor,
                offset,
                preimage,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn as_entry(&self) -> PreimageEntryRef<'_> {
        PreimageEntryRef::new_unchecked(&self.anchor, &self.preimage)
    }

    pub fn into_entry(self) -> PreimageEntry {
        PreimageEntry::new_unchecked(self.anchor, self.preimage)
    }

    pub fn into_offset_entry(self) -> (usize, PreimageEntry) {
        (self.offset, self.into_entry())
    }
}

fn as_offset(offset: U256) -> Option<usize> {
    let offset = usize::try_from(offset).ok()?;
    (offset <= MAX_STORAGE_OFFSET).then_some(offset)
}
