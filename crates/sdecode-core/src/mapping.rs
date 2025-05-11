use alloy_primitives::{B256, Bytes};
use quick_impl::quick_impl_all;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[quick_impl_all(pub const is, pub set)]
pub enum MappingKeySide {
    /// `[key][slot]`
    Left,

    /// `[slot][key]`
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MappingEntryLocation {
    pub entry_key: Bytes,
    pub mapping_slot: B256,
}

impl MappingKeySide {
    pub const SOLIDITY: Self = Self::Left;
    pub const VYPER: Self = Self::Right;

    pub const fn new(right: bool) -> Self {
        if right { Self::Right } else { Self::Left }
    }

    pub fn split(&self, preimage: impl AsRef<[u8]>) -> Option<MappingEntryLocation> {
        let preimage = preimage.as_ref();

        let key_size = preimage.len().checked_sub(32)?;

        let (key, slot) = match self {
            Self::Left => (&preimage[..key_size], &preimage[key_size..]),
            Self::Right => (&preimage[32..preimage.len()], &preimage[..32]),
        };

        debug_assert_eq!(slot.len(), 32);
        debug_assert_eq!(key.len(), key_size);

        Some(MappingEntryLocation {
            entry_key: Bytes::copy_from_slice(key),
            mapping_slot: B256::from_slice(slot),
        })
    }
}

impl From<bool> for MappingKeySide {
    fn from(right: bool) -> Self {
        Self::new(right)
    }
}

impl From<MappingKeySide> for bool {
    fn from(value: MappingKeySide) -> Self {
        value.is_right()
    }
}

impl MappingEntryLocation {
    pub fn from_preimage(side: MappingKeySide, preimage: impl AsRef<[u8]>) -> Option<Self> {
        side.split(preimage)
    }

    pub fn into_preimage(self, side: MappingKeySide) -> Bytes {
        match side {
            MappingKeySide::Left => [self.entry_key.as_ref(), self.mapping_slot.as_ref()]
                .concat()
                .into(),
            MappingKeySide::Right => [self.mapping_slot.as_ref(), self.entry_key.as_ref()]
                .concat()
                .into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{b256, bytes};

    use super::*;

    #[test]
    fn test_key_side() {
        assert_eq!(
            MappingKeySide::Left.split(bytes!(
                "0x52760d045fcb6cb07a410156f1ec0d909a3aefe6ab66a2dd898ca8e596b27a1ea0b8"
            )),
            Some(MappingEntryLocation {
                mapping_slot: b256!(
                    "0x0d045fcb6cb07a410156f1ec0d909a3aefe6ab66a2dd898ca8e596b27a1ea0b8"
                ),
                entry_key: bytes!("0x5276")
            }),
        );

        assert_eq!(
            MappingKeySide::Right.split(bytes!(
                "0x52760d045fcb6cb07a410156f1ec0d909a3aefe6ab66a2dd898ca8e596b27a1ea0b8"
            )),
            Some(MappingEntryLocation {
                mapping_slot: b256!(
                    "0x52760d045fcb6cb07a410156f1ec0d909a3aefe6ab66a2dd898ca8e596b27a1e"
                ),
                entry_key: bytes!("0xa0b8")
            }),
        );

        assert!(MappingKeySide::Right.split(bytes!("0x5276")).is_none());
    }
}
