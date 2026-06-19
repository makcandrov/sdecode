use alloy_primitives::{B256, Bytes};
use quick_impl::quick_impl_all;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[quick_impl_all(pub const is, pub set)]
pub enum MappingKeySide {
    /// `[key][slot]`
    Left,

    /// `[slot][key]`
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
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

    /// Splits an owned `preimage` into its `(entry_key, mapping_slot)` parts.
    pub fn split(&self, preimage: Bytes) -> Result<MappingEntryLocation, Bytes> {
        let Some(key_size) = preimage.len().checked_sub(32) else {
            return Err(preimage);
        };

        let mut inner = preimage.0;

        let (entry_key, mapping_slot) = match self {
            // `[key][slot]`: keep `key`, split off the trailing slot.
            Self::Left => {
                let slot = inner.split_off(key_size);
                (Bytes(inner), B256::from_slice(&slot))
            }
            // `[slot][key]`: split off the trailing key, keep the leading slot.
            Self::Right => {
                let key = inner.split_off(32);
                (Bytes(key), B256::from_slice(&inner))
            }
        };

        debug_assert_eq!(entry_key.len(), key_size);

        Ok(MappingEntryLocation {
            entry_key,
            mapping_slot,
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
    pub fn try_from_preimage(side: MappingKeySide, preimage: Bytes) -> Result<Self, Bytes> {
        side.split(preimage)
    }

    pub fn into_preimage(self, side: MappingKeySide) -> Bytes {
        let b = match side {
            MappingKeySide::Left => [self.entry_key.as_ref(), self.mapping_slot.as_ref()],
            MappingKeySide::Right => [self.mapping_slot.as_ref(), self.entry_key.as_ref()],
        };
        b.concat().into()
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
            Ok(MappingEntryLocation {
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
            Ok(MappingEntryLocation {
                mapping_slot: b256!(
                    "0x52760d045fcb6cb07a410156f1ec0d909a3aefe6ab66a2dd898ca8e596b27a1e"
                ),
                entry_key: bytes!("0xa0b8")
            }),
        );

        assert!(MappingKeySide::Right.split(bytes!("0x5276")).is_err());
    }
}
