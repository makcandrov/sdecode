#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use alloy_primitives::B256;
use std::collections::BTreeMap;

pub type StorageEntries = BTreeMap<B256, B256>;

mod decode;
pub use decode::{SdecodeMutResult, SdecodeResult, StorageDecode, StorageError};

mod item;
pub use item::{AnchorKind, HashChain, HashLink, StorageItem};

mod mapping;
pub use mapping::{MappingEntryLocation, MappingKeySide};

mod node;
pub use node::{StorageNode, StorageNodeChildren, StorageStructure};

mod reader;
use reader::StorageReaderImpl;
pub use reader::{IntoStorageReader, StorageReader, StorageReaderNext, SubB256};

mod slot;
pub use slot::{DecodedStorageSlot, MAX_STORAGE_OFFSET, MAX_STORAGE_OFFSET_U256};

mod storage;
pub use storage::Storage;

mod utils;
