use alloy_primitives::{Address, Bytes, FixedBytes, Function, aliases::*};
use sdecode_core::{IntoStorageReader, StorageReader};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    hash::{BuildHasher, Hash},
    marker::PhantomData,
};

use crate::{
    SolLayoutError, SolMappingKeyType, SolStorageType, SolStorageValue, SolWordType, sol_type,
};

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
    bool => sol_type!(bool),
    u8 => sol_type!(uint8),
    u16 => sol_type!(uint16),
    U24 => sol_type!(uint24),
    u32 => sol_type!(uint32),
    U40 => sol_type!(uint40),
    U48 => sol_type!(uint48),
    U56 => sol_type!(uint56),
    u64 => sol_type!(uint64),
    U72 => sol_type!(uint72),
    U80 => sol_type!(uint80),
    U88 => sol_type!(uint88),
    U96 => sol_type!(uint96),
    U104 => sol_type!(uint104),
    U112 => sol_type!(uint112),
    U120 => sol_type!(uint120),
    u128 => sol_type!(uint128),
    U136 => sol_type!(uint136),
    U144 => sol_type!(uint144),
    U152 => sol_type!(uint152),
    U160 => sol_type!(uint160),
    U168 => sol_type!(uint168),
    U176 => sol_type!(uint176),
    U184 => sol_type!(uint184),
    U192 => sol_type!(uint192),
    U200 => sol_type!(uint200),
    U208 => sol_type!(uint208),
    U216 => sol_type!(uint216),
    U224 => sol_type!(uint224),
    U232 => sol_type!(uint232),
    U240 => sol_type!(uint240),
    U248 => sol_type!(uint248),
    U256 => sol_type!(uint256),

    i8 => sol_type!(int8),
    i16 => sol_type!(int16),
    I24 => sol_type!(int24),
    i32 => sol_type!(int32),
    I40 => sol_type!(int40),
    I48 => sol_type!(int48),
    I56 => sol_type!(int56),
    i64 => sol_type!(int64),
    I72 => sol_type!(int72),
    I80 => sol_type!(int80),
    I88 => sol_type!(int88),
    I96 => sol_type!(int96),
    I104 => sol_type!(int104),
    I112 => sol_type!(int112),
    I120 => sol_type!(int120),
    i128 => sol_type!(int128),
    I136 => sol_type!(int136),
    I144 => sol_type!(int144),
    I152 => sol_type!(int152),
    I160 => sol_type!(int160),
    I168 => sol_type!(int168),
    I176 => sol_type!(int176),
    I184 => sol_type!(int184),
    I192 => sol_type!(int192),
    I200 => sol_type!(int200),
    I208 => sol_type!(int208),
    I216 => sol_type!(int216),
    I224 => sol_type!(int224),
    I232 => sol_type!(int232),
    I240 => sol_type!(int240),
    I248 => sol_type!(int248),
    I256 => sol_type!(int256),

    FixedBytes<1> => sol_type!(bytes1),
    FixedBytes<2> => sol_type!(bytes2),
    FixedBytes<3> => sol_type!(bytes3),
    FixedBytes<4> => sol_type!(bytes4),
    FixedBytes<5> => sol_type!(bytes5),
    FixedBytes<6> => sol_type!(bytes6),
    FixedBytes<7> => sol_type!(bytes7),
    FixedBytes<8> => sol_type!(bytes8),
    FixedBytes<9> => sol_type!(bytes9),
    FixedBytes<10> => sol_type!(bytes10),
    FixedBytes<11> => sol_type!(bytes11),
    FixedBytes<12> => sol_type!(bytes12),
    FixedBytes<13> => sol_type!(bytes13),
    FixedBytes<14> => sol_type!(bytes14),
    FixedBytes<15> => sol_type!(bytes15),
    FixedBytes<16> => sol_type!(bytes16),
    FixedBytes<17> => sol_type!(bytes17),
    FixedBytes<18> => sol_type!(bytes18),
    FixedBytes<19> => sol_type!(bytes19),
    FixedBytes<20> => sol_type!(bytes20),
    FixedBytes<21> => sol_type!(bytes21),
    FixedBytes<22> => sol_type!(bytes22),
    FixedBytes<23> => sol_type!(bytes23),
    FixedBytes<24> => sol_type!(bytes24),
    FixedBytes<25> => sol_type!(bytes25),
    FixedBytes<26> => sol_type!(bytes26),
    FixedBytes<27> => sol_type!(bytes27),
    FixedBytes<28> => sol_type!(bytes28),
    FixedBytes<29> => sol_type!(bytes29),
    FixedBytes<30> => sol_type!(bytes30),
    FixedBytes<31> => sol_type!(bytes31),
    FixedBytes<32> => sol_type!(bytes32),

    [u8; 1] => sol_type!(bytes1),
    [u8; 2] => sol_type!(bytes2),
    [u8; 3] => sol_type!(bytes3),
    [u8; 4] => sol_type!(bytes4),
    [u8; 5] => sol_type!(bytes5),
    [u8; 6] => sol_type!(bytes6),
    [u8; 7] => sol_type!(bytes7),
    [u8; 8] => sol_type!(bytes8),
    [u8; 9] => sol_type!(bytes9),
    [u8; 10] => sol_type!(bytes10),
    [u8; 11] => sol_type!(bytes11),
    [u8; 12] => sol_type!(bytes12),
    [u8; 13] => sol_type!(bytes13),
    [u8; 14] => sol_type!(bytes14),
    [u8; 15] => sol_type!(bytes15),
    [u8; 16] => sol_type!(bytes16),
    [u8; 17] => sol_type!(bytes17),
    [u8; 18] => sol_type!(bytes18),
    [u8; 19] => sol_type!(bytes19),
    [u8; 20] => sol_type!(bytes20),
    [u8; 21] => sol_type!(bytes21),
    [u8; 22] => sol_type!(bytes22),
    [u8; 23] => sol_type!(bytes23),
    [u8; 24] => sol_type!(bytes24),
    [u8; 25] => sol_type!(bytes25),
    [u8; 26] => sol_type!(bytes26),
    [u8; 27] => sol_type!(bytes27),
    [u8; 28] => sol_type!(bytes28),
    [u8; 29] => sol_type!(bytes29),
    [u8; 30] => sol_type!(bytes30),
    [u8; 31] => sol_type!(bytes31),
    [u8; 32] => sol_type!(bytes32),

    Address => sol_type!(address),
    Function => sol_type!(function()),
];

impl SolMappingKeyValue<sol_type!(bytes)> for Bytes {
    fn into_sol_mapping_key(self) -> Bytes {
        self
    }

    fn try_from_sol_mapping_key(key: Bytes) -> Result<Self, Bytes> {
        Ok(key)
    }
}

impl SolMappingKeyValue<sol_type!(string)> for String {
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

impl<K, SolK, V, SolV, M> SolStorageValue<sol_type!(mapping(SolK => SolV))>
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
                sol_type: <sol_type!(mapping(SolK => SolV))>::SOL_STORAGE_NAME,
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

impl<K, SolK, V, SolV> SolStorageValue<sol_type!(mapping(SolK => SolV))> for BTreeMap<K, V>
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
        <SolMappingHelper<Self, K, V> as SolStorageValue<sol_type!(mapping(SolK => SolV))>>::decode_storage(
            storage_reader,
        )
        .map(|x| x.0)
    }
}

impl<K, SolK, V, SolV, S> SolStorageValue<sol_type!(mapping(SolK => SolV))> for HashMap<K, V, S>
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
        <SolMappingHelper<Self, K, V> as SolStorageValue<sol_type!(mapping(SolK => SolV))>>::decode_storage(
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

impl<K, SolK, M> SolStorageValue<sol_type!(mapping(SolK => bool))> for SolSetHelper<M, K>
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
                sol_type: <sol_type!(mapping(SolK => bool))>::SOL_STORAGE_NAME,
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
                match <bool as SolStorageValue<sol_type!(bool)>>::decode_storage(
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

impl<K, SolK> SolStorageValue<sol_type!(mapping(SolK => bool))> for BTreeSet<K>
where
    K: SolMappingKeyValue<SolK> + Ord,
    SolK: SolMappingKeyType,
{
    fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
    where
        Reader: StorageReader,
    {
        <SolSetHelper<Self, K> as SolStorageValue<
            sol_type!(mapping(SolK => bool)),
        >>::decode_storage(storage_reader)
        .map(|x| x.0)
    }
}

impl<K, SolK, S> SolStorageValue<sol_type!(mapping(SolK => bool))> for HashSet<K, S>
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
            sol_type!(mapping(SolK => bool)),
        >>::decode_storage(storage_reader)
        .map(|x| x.0)
    }
}

#[cfg(feature = "hashbrown")]
const _: () = {
    use hashbrown::{HashMap, HashSet};

    impl<K, SolK, V, SolV, S> SolStorageValue<sol_type!(mapping(SolK => SolV))> for HashMap<K, V, S>
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
            <SolMappingHelper<Self, K, V> as SolStorageValue<sol_type!(mapping(SolK => SolV))>>::decode_storage(
                storage_reader,
            )
            .map(|x| x.0)
        }
    }

    impl<K, SolK, S> SolStorageValue<sol_type!(mapping(SolK => bool))> for HashSet<K, S>
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
                sol_type!(mapping(SolK => bool)),
            >>::decode_storage(storage_reader)
            .map(|x| x.0)
        }
    }
};

#[cfg(feature = "indexmap")]
const _: () = {
    use indexmap::{IndexMap, IndexSet};

    impl<K, SolK, V, SolV, S> SolStorageValue<sol_type!(mapping(SolK => SolV))> for IndexMap<K, V, S>
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
            <SolMappingHelper<Self, K, V> as SolStorageValue<sol_type!(mapping(SolK => SolV))>>::decode_storage(
                storage_reader,
            )
            .map(|x| x.0)
        }
    }

    impl<K, SolK, S> SolStorageValue<sol_type!(mapping(SolK => bool))> for IndexSet<K, S>
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
                sol_type!(mapping(SolK => bool)),
            >>::decode_storage(storage_reader)
            .map(|x| x.0)
        }
    }
};
