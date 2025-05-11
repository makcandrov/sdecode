use alloy_primitives::{B256, Bytes, U256};
use quick_impl::quick_impl;
use sdecode_preimages::{PreimagesProvider, PreimagesProviderMut, WrapPreimagesProvider};

use crate::utils::b256_to_u256;

pub const MAX_STORAGE_OFFSET: usize = 0xffffffffffff;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[quick_impl(pub const new = "new_unchecked", pub into_parts = "split")]
pub struct DecodedStorageSlot {
    #[quick_impl(pub get_clone = "{}")]
    slot: B256,

    #[quick_impl(pub get_clone = "{}")]
    offset: usize,

    #[quick_impl(pub get = "{}")]
    preimage: Bytes,
}

impl DecodedStorageSlot {
    pub fn decode<P: PreimagesProvider>(provider: P, slot: B256) -> Result<Option<Self>, P::Error> {
        Self::decode_mut(&mut WrapPreimagesProvider(provider), slot)
    }

    pub fn decode_mut<P: PreimagesProviderMut>(
        provider: &mut P,
        slot: B256,
    ) -> Result<Option<Self>, P::Error> {
        let Some(entry) = provider.nearest_lower_preimage_mut(slot)? else {
            return Ok(None);
        };

        let (image, preimage) = entry.into_parts();

        let offset = b256_to_u256(slot)
            .checked_sub(b256_to_u256(image))
            .expect("should be lower");

        if let Some(offset) = as_offset(offset) {
            Ok(Some(Self {
                slot: image,
                offset,
                preimage,
            }))
        } else {
            Ok(None)
        }
    }
}

fn as_offset(offset: U256) -> Option<usize> {
    let offset = usize::try_from(offset).ok()?;
    (offset <= MAX_STORAGE_OFFSET).then_some(offset)
}
