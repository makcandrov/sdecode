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
    pub use super::dynamic_array::SolDynamicArrayHelper;
    pub use super::enumm::SolEnumHelper;
    pub use super::fixed_array::SolFixedArrayHelper;
    pub use super::mapping::{SolMappingHelper, SolSetHelper};
    pub use super::structure::SolStructureHelper;
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

    /// The slot has child entries (sub-keys) but the expected type is a leaf (e.g. a word type,
    /// a short bytes/string, or a bytes/string data chunk). This means the storage data contains
    /// nested structure where the layout expects a simple value.
    #[error("unexpected children in slot: expected a leaf value but found {count} child entries")]
    UnexpectedChildren { count: usize },

    /// A dynamic array's stored length is too large to fit in a u64.
    #[error("dynamic array length too large: {value}")]
    ArrayLengthOverflow { value: B256 },

    /// The raw bytes in a packed word could not be decoded into the expected Solidity type.
    #[error("invalid packed word value: {word}")]
    InvalidWordValue { word: Bytes },

    /// An enum discriminant does not correspond to any known variant.
    #[error("invalid enum discriminant: {discriminant}")]
    InvalidEnumDiscriminant { discriminant: u8 },
}

impl SolLayoutError {
    pub fn remaining_bytes(remaining: impl Into<Bytes>) -> Self {
        Self::RemainingBytes {
            remaining: remaining.into(),
        }
    }
}
