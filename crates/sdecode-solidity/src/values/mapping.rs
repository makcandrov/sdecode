use alloy_primitives::{Address, Bytes, FixedBytes, Function, aliases::*};
use sdecode_core::{IntoStorageReader, StorageReader};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    hash::{BuildHasher, Hash},
    marker::PhantomData,
};

use crate::{SolMappingKeyType, SolStorageType, data_types};

use super::{SolLayoutError, SolStorageValue, word::SolWordType};

/// # [Mappings and Dynamic Arrays](https://docs.soliditylang.org/en/latest/internals/layout_in_storage.html#mappings-and-dynamic-arrays)
///
/// The value corresponding to a mapping key ``k`` is located at ``keccak256(h(k) . p)``
/// where ``.`` is concatenation and ``h`` is a function that is applied to the key depending on its
/// type:
///
/// - for value types, ``h`` pads the value to 32 bytes in the same way as when storing the value in
///   memory.
/// - for strings and byte arrays, ``h(k)`` is just the unpadded data.
pub trait SolMappingKeyValue<SolK: SolMappingKeyType>: Sized {
    fn into_sol_mapping_key(self) -> Bytes;
    fn try_from_sol_mapping_key(key: Bytes) -> Result<Self, Bytes>;
}

macro_rules! impl_sol_mapping_key_value_for_word {
    ($($t:ty => $sol_t:ty),* $(,)?) => {
        $(
            impl SolMappingKeyValue<$sol_t> for $t {
                fn into_sol_mapping_key(self) -> Bytes {
                    self.into_word().into()
                }

                fn try_from_sol_mapping_key(key: Bytes) -> Result<Self, Bytes> {
                    if key.len() != 32 {
                        return Err(key);
                    }
                    <$t>::try_from_word(B256::from_slice(&key)).ok_or(key)
                }
            }
        )*
    };
}

impl_sol_mapping_key_value_for_word![
    bool => data_types::Bool,
    u8 => data_types::Uint<8>,
    u16 => data_types::Uint<16>,
    U24 => data_types::Uint<24>,
    u32 => data_types::Uint<32>,
    U40 => data_types::Uint<40>,
    U48 => data_types::Uint<48>,
    U56 => data_types::Uint<56>,
    u64 => data_types::Uint<64>,
    U72 => data_types::Uint<72>,
    U80 => data_types::Uint<80>,
    U88 => data_types::Uint<88>,
    U96 => data_types::Uint<96>,
    U104 => data_types::Uint<104>,
    U112 => data_types::Uint<112>,
    U120 => data_types::Uint<120>,
    u128 => data_types::Uint<128>,
    U136 => data_types::Uint<136>,
    U144 => data_types::Uint<144>,
    U152 => data_types::Uint<152>,
    U160 => data_types::Uint<160>,
    U168 => data_types::Uint<168>,
    U176 => data_types::Uint<176>,
    U184 => data_types::Uint<184>,
    U192 => data_types::Uint<192>,
    U200 => data_types::Uint<200>,
    U208 => data_types::Uint<208>,
    U216 => data_types::Uint<216>,
    U224 => data_types::Uint<224>,
    U232 => data_types::Uint<232>,
    U240 => data_types::Uint<240>,
    U248 => data_types::Uint<248>,
    U256 => data_types::Uint<256>,

    i8 => data_types::Int<8>,
    i16 => data_types::Int<16>,
    I24 => data_types::Int<24>,
    i32 => data_types::Int<32>,
    I40 => data_types::Int<40>,
    I48 => data_types::Int<48>,
    I56 => data_types::Int<56>,
    i64 => data_types::Int<64>,
    I72 => data_types::Int<72>,
    I80 => data_types::Int<80>,
    I88 => data_types::Int<88>,
    I96 => data_types::Int<96>,
    I104 => data_types::Int<104>,
    I112 => data_types::Int<112>,
    I120 => data_types::Int<120>,
    i128 => data_types::Int<128>,
    I136 => data_types::Int<136>,
    I144 => data_types::Int<144>,
    I152 => data_types::Int<152>,
    I160 => data_types::Int<160>,
    I168 => data_types::Int<168>,
    I176 => data_types::Int<176>,
    I184 => data_types::Int<184>,
    I192 => data_types::Int<192>,
    I200 => data_types::Int<200>,
    I208 => data_types::Int<208>,
    I216 => data_types::Int<216>,
    I224 => data_types::Int<224>,
    I232 => data_types::Int<232>,
    I240 => data_types::Int<240>,
    I248 => data_types::Int<248>,
    I256 => data_types::Int<256>,

    FixedBytes<1> => data_types::FixedBytes<1>,
    FixedBytes<2> => data_types::FixedBytes<2>,
    FixedBytes<3> => data_types::FixedBytes<3>,
    FixedBytes<4> => data_types::FixedBytes<4>,
    FixedBytes<5> => data_types::FixedBytes<5>,
    FixedBytes<6> => data_types::FixedBytes<6>,
    FixedBytes<7> => data_types::FixedBytes<7>,
    FixedBytes<8> => data_types::FixedBytes<8>,
    FixedBytes<9> => data_types::FixedBytes<9>,
    FixedBytes<10> => data_types::FixedBytes<10>,
    FixedBytes<11> => data_types::FixedBytes<11>,
    FixedBytes<12> => data_types::FixedBytes<12>,
    FixedBytes<13> => data_types::FixedBytes<13>,
    FixedBytes<14> => data_types::FixedBytes<14>,
    FixedBytes<15> => data_types::FixedBytes<15>,
    FixedBytes<16> => data_types::FixedBytes<16>,
    FixedBytes<17> => data_types::FixedBytes<17>,
    FixedBytes<18> => data_types::FixedBytes<18>,
    FixedBytes<19> => data_types::FixedBytes<19>,
    FixedBytes<20> => data_types::FixedBytes<20>,
    FixedBytes<21> => data_types::FixedBytes<21>,
    FixedBytes<22> => data_types::FixedBytes<22>,
    FixedBytes<23> => data_types::FixedBytes<23>,
    FixedBytes<24> => data_types::FixedBytes<24>,
    FixedBytes<25> => data_types::FixedBytes<25>,
    FixedBytes<26> => data_types::FixedBytes<26>,
    FixedBytes<27> => data_types::FixedBytes<27>,
    FixedBytes<28> => data_types::FixedBytes<28>,
    FixedBytes<29> => data_types::FixedBytes<29>,
    FixedBytes<30> => data_types::FixedBytes<30>,
    FixedBytes<31> => data_types::FixedBytes<31>,
    FixedBytes<32> => data_types::FixedBytes<32>,

    [u8; 1] => data_types::FixedBytes<1>,
    [u8; 2] => data_types::FixedBytes<2>,
    [u8; 3] => data_types::FixedBytes<3>,
    [u8; 4] => data_types::FixedBytes<4>,
    [u8; 5] => data_types::FixedBytes<5>,
    [u8; 6] => data_types::FixedBytes<6>,
    [u8; 7] => data_types::FixedBytes<7>,
    [u8; 8] => data_types::FixedBytes<8>,
    [u8; 9] => data_types::FixedBytes<9>,
    [u8; 10] => data_types::FixedBytes<10>,
    [u8; 11] => data_types::FixedBytes<11>,
    [u8; 12] => data_types::FixedBytes<12>,
    [u8; 13] => data_types::FixedBytes<13>,
    [u8; 14] => data_types::FixedBytes<14>,
    [u8; 15] => data_types::FixedBytes<15>,
    [u8; 16] => data_types::FixedBytes<16>,
    [u8; 17] => data_types::FixedBytes<17>,
    [u8; 18] => data_types::FixedBytes<18>,
    [u8; 19] => data_types::FixedBytes<19>,
    [u8; 20] => data_types::FixedBytes<20>,
    [u8; 21] => data_types::FixedBytes<21>,
    [u8; 22] => data_types::FixedBytes<22>,
    [u8; 23] => data_types::FixedBytes<23>,
    [u8; 24] => data_types::FixedBytes<24>,
    [u8; 25] => data_types::FixedBytes<25>,
    [u8; 26] => data_types::FixedBytes<26>,
    [u8; 27] => data_types::FixedBytes<27>,
    [u8; 28] => data_types::FixedBytes<28>,
    [u8; 29] => data_types::FixedBytes<29>,
    [u8; 30] => data_types::FixedBytes<30>,
    [u8; 31] => data_types::FixedBytes<31>,
    [u8; 32] => data_types::FixedBytes<32>,

    Address => data_types::Address,
    Function => data_types::Function,
];

impl SolMappingKeyValue<data_types::Bytes> for Bytes {
    fn into_sol_mapping_key(self) -> Bytes {
        self
    }

    fn try_from_sol_mapping_key(key: Bytes) -> Result<Self, Bytes> {
        Ok(key)
    }
}

impl SolMappingKeyValue<data_types::String> for String {
    fn into_sol_mapping_key(self) -> Bytes {
        self.into_bytes().into()
    }

    fn try_from_sol_mapping_key(key: Bytes) -> Result<Self, Bytes> {
        Ok(Self::from_utf8_lossy(key.as_ref()).into_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SolMappingHelper<M, K, V>(pub M, PhantomData<(K, V)>);

impl<M, K, V> SolMappingHelper<M, K, V> {
    pub const fn new(mapping: M) -> Self {
        Self(mapping, PhantomData)
    }
}

impl<K, SolK, V, SolV, M> SolStorageValue<data_types::Mapping<SolK, SolV>>
    for SolMappingHelper<M, K, V>
where
    K: SolMappingKeyValue<SolK>,
    V: SolStorageValue<SolV>,
    SolK: SolMappingKeyType,
    SolV: SolStorageType,
    M: FromIterator<(K, V)>,
{
    fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
    where
        Reader: StorageReader,
    {
        let next = storage_reader.next_or_default::<B256>();

        if next.is_remaining_not_zero() {
            return Err(SolLayoutError::remaining_bytes(next.remaining));
        }

        if !next.word.is_zero() {
            return Err(SolLayoutError::NonEmptySlot {
                sol_type: data_types::Mapping::<SolK, SolV>::SOL_STORAGE_NAME,
                value: next.word,
            });
        }

        next.children
            .into_iter()
            .map(|(key, structure)| -> Result<_, SolLayoutError> {
                let key = K::try_from_sol_mapping_key(key).map_err(|raw| {
                    SolLayoutError::InvalidMappingKey {
                        sol_type: SolK::SOL_STORAGE_NAME,
                        raw,
                    }
                })?;
                let value = V::decode_storage(&mut structure.into_storage_reader())?;
                Ok((key, value))
            })
            .collect::<Result<M, _>>()
            .map(Self::new)
    }
}

impl<K, SolK, V, SolV> SolStorageValue<data_types::Mapping<SolK, SolV>> for BTreeMap<K, V>
where
    K: SolMappingKeyValue<SolK> + Ord,
    V: SolStorageValue<SolV>,
    SolK: SolMappingKeyType,
    SolV: SolStorageType,
{
    fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
    where
        Reader: StorageReader,
    {
        <SolMappingHelper<Self, K, V> as SolStorageValue<data_types::Mapping<SolK, SolV>>>::decode_storage(
            storage_reader,
        )
        .map(|x| x.0)
    }
}

impl<K, SolK, V, SolV, S> SolStorageValue<data_types::Mapping<SolK, SolV>> for HashMap<K, V, S>
where
    S: BuildHasher + Default,
    K: SolMappingKeyValue<SolK> + Eq + Hash,
    V: SolStorageValue<SolV>,
    SolK: SolMappingKeyType,
    SolV: SolStorageType,
{
    fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
    where
        Reader: StorageReader,
    {
        <SolMappingHelper<Self, K, V> as SolStorageValue<data_types::Mapping<SolK, SolV>>>::decode_storage(
            storage_reader,
        )
        .map(|x| x.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SolSetHelper<M, K>(pub M, PhantomData<K>);

impl<M, K> SolSetHelper<M, K> {
    pub const fn new(mapping: M) -> Self {
        Self(mapping, PhantomData)
    }
}

impl<K, SolK, M> SolStorageValue<data_types::Mapping<SolK, data_types::Bool>> for SolSetHelper<M, K>
where
    K: SolMappingKeyValue<SolK>,
    SolK: SolMappingKeyType,
    M: FromIterator<K>,
{
    fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
    where
        Reader: StorageReader,
    {
        let next = storage_reader.next_or_default::<B256>();

        if next.is_remaining_not_zero() {
            return Err(SolLayoutError::remaining_bytes(next.remaining));
        }

        if !next.word.is_zero() {
            return Err(SolLayoutError::NonEmptySlot {
                sol_type: data_types::Mapping::<SolK, data_types::Bool>::SOL_STORAGE_NAME,
                value: next.word,
            });
        }

        next.children
            .into_iter()
            .filter_map(|(key, structure)| -> Option<Result<_, SolLayoutError>> {
                let key = match K::try_from_sol_mapping_key(key) {
                    Ok(key) => key,
                    Err(raw) => {
                        return Some(Err(SolLayoutError::InvalidMappingKey {
                            sol_type: SolK::SOL_STORAGE_NAME,
                            raw,
                        }));
                    }
                };
                match <bool as SolStorageValue<data_types::Bool>>::decode_storage(
                    &mut structure.into_storage_reader(),
                ) {
                    Ok(present) => present.then_some(Ok(key)),
                    Err(err) => Some(Err(err)),
                }
            })
            .collect::<Result<M, _>>()
            .map(Self::new)
    }
}

impl<K, SolK> SolStorageValue<data_types::Mapping<SolK, data_types::Bool>> for BTreeSet<K>
where
    K: SolMappingKeyValue<SolK> + Ord,
    SolK: SolMappingKeyType,
{
    fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
    where
        Reader: StorageReader,
    {
        <SolSetHelper<Self, K> as SolStorageValue<
            data_types::Mapping<SolK, data_types::Bool>,
        >>::decode_storage(storage_reader)
        .map(|x| x.0)
    }
}

impl<K, SolK, S> SolStorageValue<data_types::Mapping<SolK, data_types::Bool>> for HashSet<K, S>
where
    S: BuildHasher + Default,
    K: SolMappingKeyValue<SolK> + Eq + Hash,
    SolK: SolMappingKeyType,
{
    fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
    where
        Reader: StorageReader,
    {
        <SolSetHelper<Self, K> as SolStorageValue<
            data_types::Mapping<SolK, data_types::Bool>,
        >>::decode_storage(storage_reader)
        .map(|x| x.0)
    }
}
