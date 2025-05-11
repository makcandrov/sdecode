use crate::{
    data_types,
    utils::{b256_to_u256, slice_is_zero},
};

use super::{SolLayoutError, SolStorageValue};
use alloy_primitives::{
    B256, Bytes,
    bytes::{self, BufMut, BytesMut},
    uint,
};
use overf::checked;
use sdecode_core::{IntoStorageReader, StorageReader, StorageReaderNext};

impl SolStorageValue<data_types::Bytes> for bytes::Bytes {
    /// # [`bytes` and `string`](https://docs.soliditylang.org/en/latest/internals/layout_in_storage.html#bytes-and-string)
    ///
    /// ``bytes`` and ``string`` are encoded identically.
    /// In general, the encoding is similar to ``bytes1[]``, in the sense that there is a slot for
    /// the array itself and a data area that is computed using a ``keccak256`` hash of that
    /// slot's position. However, for short values (shorter than 32 bytes) the array elements
    /// are stored together with the length in the same slot.
    ///
    /// In particular: if the data is at most ``31`` bytes long, the elements are stored
    /// in the higher-order bytes (left aligned) and the lowest-order byte stores the value ``length
    /// * 2``. For byte arrays that store data which is ``32`` or more bytes long, the main slot
    /// ``p`` stores ``length * 2 + 1`` and the data is stored as usual in ``keccak256(p)``.
    ///
    /// This means that you can distinguish a short array from a long array by checking if the
    /// lowest bit is set: short (not set) and long (set).
    fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
    where
        Reader: StorageReader,
    {
        let remaining = storage_reader.consume_remaining();

        if remaining.is_not_zero() {
            return Err(SolLayoutError::remaining_bytes(remaining));
        }

        let next = storage_reader.next_or_default::<B256>();

        if next.is_remaining_not_zero() {
            return Err(SolLayoutError::remaining_bytes(remaining));
        }

        let StorageReaderNext {
            word, mut children, ..
        } = next;

        let last_byte = word[31];
        let is_short = last_byte % 2 == 0;
        if is_short {
            if !children.is_empty() {
                return Err(SolLayoutError::Err);
            }
            let size = (last_byte / 2) as usize;
            let remaining = &word[size..31];
            if slice_is_zero(remaining) {
                Ok(Self::copy_from_slice(&word[0..size]))
            } else {
                Err(SolLayoutError::remaining_bytes(Bytes::copy_from_slice(
                    remaining,
                )))
            }
        } else {
            let size = checked! { (b256_to_u256(word) - uint!(1_U256)) / uint!(2_U256) };
            let mut size = u64::try_from(size).unwrap();

            let child = children.remove(&Bytes::new()).unwrap_or_default();

            if !children.is_empty() {
                return Err(SolLayoutError::Err);
            }

            let mut child_storage_reader = child.into_storage_reader();

            let mut buf = BytesMut::new();
            while let Some(new_size) = size.checked_sub(32) {
                let next = child_storage_reader.next_or_default::<B256>();

                if !next.is_remaining_zero() {
                    return Err(SolLayoutError::Err);
                }

                let StorageReaderNext {
                    word: chunk,
                    children,
                    ..
                } = next;

                if !children.is_empty() {
                    return Err(SolLayoutError::Err);
                }
                buf.put_slice(chunk.as_ref());
                size = new_size;
            }

            if size > 0 {
                let next = child_storage_reader.next_or_default::<B256>();

                if !next.is_remaining_zero() {
                    return Err(SolLayoutError::Err);
                }

                let StorageReaderNext {
                    word: chunk,
                    children,
                    ..
                } = next;

                if !children.is_empty() {
                    return Err(SolLayoutError::Err);
                }
                buf.put_slice(&chunk[0..(size as usize)]);

                let remaining = &chunk[(size as usize)..];

                if !slice_is_zero(remaining) {
                    return Err(SolLayoutError::Err);
                }
            }

            Ok(buf.freeze())
        }
    }
}

impl SolStorageValue<data_types::Bytes> for Bytes {
    fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
    where
        Reader: StorageReader,
    {
        let b = bytes::Bytes::decode_storage(storage_reader)?;
        Ok(b.into())
    }
}

impl SolStorageValue<data_types::Bytes> for Vec<u8> {
    fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
    where
        Reader: StorageReader,
    {
        let b = bytes::Bytes::decode_storage(storage_reader)?;
        Ok(b.into())
    }
}

impl SolStorageValue<data_types::String> for String {
    fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
    where
        Reader: StorageReader,
    {
        let b = Bytes::decode_storage(storage_reader)?;
        Ok(Self::from_utf8_lossy(&b).into_owned())
    }
}
