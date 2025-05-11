use alloy_primitives::{B256, Bytes};
use sdecode_core::StorageReader;

use crate::SolStorageType;

mod bytes_string;

mod enumm;

mod dynamic_array;

mod fixed_array;

mod mapping;
pub use mapping::SolMappingKeyValue;

mod structure;

mod word;
pub use word::SolWordType;

pub mod helpers {
    use super::*;

    pub use dynamic_array::SolDynamicArrayHelper;
    pub use enumm::SolEnumHelper;
    pub use fixed_array::SolFixedArrayHelper;
    pub use mapping::{SolMappingHelper, SolSetHelper};
    pub use structure::SolStructureHelper;
}

pub trait SolStorageValue<T: SolStorageType>: Sized {
    fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
    where
        Reader: StorageReader;
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum SolLayoutError {
    /// Triggered when a storage entry has non-zero bytes on an unused part.
    ///
    /// For example, an address uses only 20 bytes in storage. So in the following contract
    ///
    /// ```solidity
    /// contract Contract {
    ///     address addr;
    ///     uint256 value;
    /// }
    /// ```
    ///
    /// The 12 fist bytes of the first slot must remain zero.
    #[error("non zero bytes remaining on unused part of a word: {remaining}")]
    RemainingBytes { remaining: Bytes },

    #[error("invalid mapping key, expected {sol_type} got {raw}")]
    InvalidMappingKey { sol_type: &'static str, raw: Bytes },

    /// When decoding a mapping or a dynamic array, the slot of the variable must be empty.
    #[error("expected empty slot, got {value}")]
    NonEmptySlot { sol_type: &'static str, value: B256 },

    #[error("todo error")]
    Err,
}

impl SolLayoutError {
    pub fn remaining_bytes(remaining: impl Into<Bytes>) -> Self {
        Self::RemainingBytes {
            remaining: remaining.into(),
        }
    }
}
