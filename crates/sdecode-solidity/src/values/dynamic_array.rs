use std::marker::PhantomData;

use alloy_primitives::{B256, Bytes};
use sdecode_core::{IntoStorageReader, StorageReader, StorageReaderNext};

use crate::{SolStorageType, data_types, utils::b256_to_u256};

use super::{SolLayoutError, SolStorageValue};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SolDynamicArrayHelper<A, T>(pub A, PhantomData<T>);

impl<A, T> SolDynamicArrayHelper<A, T> {
    pub const fn new(array: A) -> Self {
        Self(array, PhantomData)
    }
}

impl<A, T, SolT> SolStorageValue<data_types::Array<SolT>> for SolDynamicArrayHelper<A, T>
where
    A: FromIterator<T>,
    T: SolStorageValue<SolT>,
    SolT: SolStorageType,
{
    fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
    where
        Reader: StorageReader,
    {
        let StorageReaderNext {
            word,
            mut children,
            remaining,
        } = storage_reader.next_or_default::<B256>();

        if remaining.is_not_zero() {
            return Err(SolLayoutError::remaining_bytes(remaining));
        }

        let Ok(size) = u64::try_from(b256_to_u256(word)) else {
            return Err(SolLayoutError::Err);
        };

        let child = children.remove(&Bytes::new()).unwrap_or_default();
        if !children.is_empty() {
            return Err(SolLayoutError::Err);
        }

        let mut child_storage_reader = child.into_storage_reader();
        (0..size)
            .map(|_| T::decode_storage(&mut child_storage_reader))
            .collect::<Result<A, _>>()
            .map(Self::new)
    }
}

impl<T, SolT> SolStorageValue<data_types::Array<SolT>> for Vec<T>
where
    T: SolStorageValue<SolT>,
    SolT: SolStorageType,
{
    fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
    where
        Reader: StorageReader,
    {
        SolDynamicArrayHelper::decode_storage(storage_reader).map(|x| x.0)
    }
}
