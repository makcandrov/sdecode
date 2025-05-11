use std::marker::PhantomData;

use sdecode_core::StorageReader;

use crate::{SolStorageType, data_types};

use super::{SolLayoutError, SolStorageValue};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SolFixedArrayHelper<const N: usize, A, T>(pub A, PhantomData<T>);

impl<const N: usize, A, T> SolFixedArrayHelper<N, A, T> {
    pub const fn new(array: A) -> Self {
        Self(array, PhantomData)
    }
}

impl<const N: usize, T, SolT> SolStorageValue<data_types::FixedArray<SolT, N>> for [T; N]
where
    SolT: SolStorageType,
    T: SolStorageValue<SolT>,
{
    fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
    where
        Reader: StorageReader,
    {
        let remaining = storage_reader.consume_remaining();

        if remaining.is_not_zero() {
            return Err(SolLayoutError::remaining_bytes(remaining));
        }

        let res = array_init::try_array_init(|_| T::decode_storage(storage_reader))?;
        Ok(res)
    }
}

impl<const N: usize, A, T, SolT> SolStorageValue<data_types::FixedArray<SolT, N>>
    for SolFixedArrayHelper<N, A, T>
where
    A: FromIterator<T>,
    SolT: SolStorageType,
    T: SolStorageValue<SolT>,
{
    fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
    where
        Reader: StorageReader,
    {
        let remaining = storage_reader.consume_remaining();

        if remaining.is_not_zero() {
            return Err(SolLayoutError::remaining_bytes(remaining));
        }

        (0..N)
            .map(|_| T::decode_storage(storage_reader))
            .collect::<Result<A, _>>()
            .map(Self::new)
    }
}

impl<const N: usize, T, SolT> SolStorageValue<data_types::FixedArray<SolT, N>> for Vec<T>
where
    SolT: SolStorageType,
    T: SolStorageValue<SolT>,
{
    fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
    where
        Reader: StorageReader,
    {
        SolFixedArrayHelper::<N, Self, T>::decode_storage(storage_reader).map(|x| x.0)
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{FixedBytes, b256, fixed_bytes};
    use sdecode_core::{IntoStorageReader, StorageNode, StorageStructure};

    use super::*;

    #[test]
    fn test_sol_fixed_size_array() {
        let structure = StorageStructure(vec![
            StorageNode::word(b256!(
                "0x0000000000000000000000030000000000000000000200000000000000000001"
            )),
            StorageNode::word(b256!(
                "0x0000000000000000000000060000000000000000000500000000000000000004"
            )),
        ]);

        let arr =
            <[FixedBytes<10>; 6]>::decode_storage(&mut structure.into_storage_reader()).unwrap();

        assert_eq!(
            arr,
            [
                fixed_bytes!("0x00000000000000000001"),
                fixed_bytes!("0x00000000000000000002"),
                fixed_bytes!("0x00000000000000000003"),
                fixed_bytes!("0x00000000000000000004"),
                fixed_bytes!("0x00000000000000000005"),
                fixed_bytes!("0x00000000000000000006"),
            ]
        );
    }
}
