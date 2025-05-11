use std::{convert::Infallible, path::Path};

use alloy_primitives::{Address, BlockNumber, ChainId};
use sdecode::{StorageDecode, StorageEntries, StorageError, preimages::MemoryPreimagesProvider};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SdecodeTestContract {
    pub chain_id: ChainId,
    pub address: Address,
    pub block: BlockNumber,
    pub storage: StorageEntries,
    pub preimages: MemoryPreimagesProvider,
}

impl SdecodeTestContract {
    pub fn decode<T: StorageDecode>(self) -> Result<T, StorageError<Infallible, T::LayoutError>> {
        T::sdecode(self.preimages, self.storage)
    }
}

pub trait JsonUtils: serde::Serialize + for<'de> serde::de::Deserialize<'de> {
    fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    fn from_json_file(path: impl AsRef<Path>) -> Result<Self, serde_json::Error> {
        let s = std::fs::read_to_string(path).unwrap();
        Self::from_json(&s)
    }

    fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    fn to_json_file(&self, path: &str) -> std::io::Result<()> {
        std::fs::write(path, self.to_json())
    }
}

impl<T: serde::Serialize + for<'de> serde::de::Deserialize<'de>> JsonUtils for T {}
