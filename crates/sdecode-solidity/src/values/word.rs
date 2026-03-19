use ::alloy_primitives::{Address, Bytes, FixedBytes, Function, aliases::*, keccak256};
use alloy_sol_types::SolValue;
use sdecode_core::{StorageReader, SubB256};

use crate::{SolLayoutError, SolStorageValue, sol_type, utils::b256_to_u256};

/// A Solidity type that fits in a 32-bytes EVM word.
pub trait SolWordType: Sized {
    const PACKED_SIZE: usize;
    type PackedBytes: SubB256;

    fn into_word(self) -> B256;
    fn try_from_word(word: B256) -> Option<Self>;

    fn into_packed_word(self) -> Self::PackedBytes;
    fn try_from_packed_word(packed_word: Self::PackedBytes) -> Option<Self>;

    fn into_word_u256(self) -> U256 {
        b256_to_u256(self.into_word())
    }

    fn into_word_keccak(self) -> B256 {
        keccak256(self.into_word())
    }

    fn into_word_with_offset(self, offset: usize) -> Option<B256> {
        let offset = U256::from(offset);
        self.into_word_u256().checked_add(offset).map(B256::from)
    }
}

// Necessary because `u8` doesn't implement `SolValue`.
impl SolWordType for u8 {
    const PACKED_SIZE: usize = 1;
    type PackedBytes = FixedBytes<{ Self::PACKED_SIZE }>;

    fn into_word(self) -> B256 {
        B256::left_padding_from(&[self])
    }

    fn try_from_word(word: B256) -> Option<Self> {
        Self::try_from(<u16 as SolValue>::abi_decode(word.as_ref()).ok()?).ok()
    }

    fn into_packed_word(self) -> Self::PackedBytes {
        FixedBytes([self])
    }

    fn try_from_packed_word(FixedBytes([packed_word]): Self::PackedBytes) -> Option<Self> {
        Some(packed_word)
    }

    fn into_word_u256(self) -> U256 {
        U256::from(self)
    }
}

macro_rules! impl_sol_word_type_for_fixed_bytes {
    ($($t:ty => $size:expr),* $(,)?) => {
        $(impl SolWordType for $t {
            const PACKED_SIZE: usize = $size;
            type PackedBytes = FixedBytes<{Self::PACKED_SIZE}>;

            fn into_word(self) -> B256 {
                B256::left_padding_from(&SolValue::abi_encode(&self))
            }

            fn try_from_word(word: B256) -> Option<Self> {
                <Self as SolValue>::abi_decode(word.as_ref()).ok()
            }

            fn into_packed_word(self) -> Self::PackedBytes {
                FixedBytes::from_slice(&<Self as SolValue>::abi_encode_packed(&self))
            }

            fn try_from_packed_word(packed_word: Self::PackedBytes) -> Option<Self> {
                let unpacked = B256::right_padding_from(packed_word.as_ref());
                Self::try_from_word(unpacked)
            }
        })?
    };
}

impl_sol_word_type_for_fixed_bytes! [
    FixedBytes<1> => 1,
    FixedBytes<2> => 2,
    FixedBytes<3> => 3,
    FixedBytes<4> => 4,
    FixedBytes<5> => 5,
    FixedBytes<6> => 6,
    FixedBytes<7> => 7,
    FixedBytes<8> => 8,
    FixedBytes<9> => 9,
    FixedBytes<10> => 10,
    FixedBytes<11> => 11,
    FixedBytes<12> => 12,
    FixedBytes<13> => 13,
    FixedBytes<14> => 14,
    FixedBytes<15> => 15,
    FixedBytes<16> => 16,
    FixedBytes<17> => 17,
    FixedBytes<18> => 18,
    FixedBytes<19> => 19,
    FixedBytes<20> => 20,
    FixedBytes<21> => 21,
    FixedBytes<22> => 22,
    FixedBytes<23> => 23,
    FixedBytes<24> => 24,
    FixedBytes<25> => 25,
    FixedBytes<26> => 26,
    FixedBytes<27> => 27,
    FixedBytes<28> => 28,
    FixedBytes<29> => 29,
    FixedBytes<30> => 30,
    FixedBytes<31> => 31,
    FixedBytes<32> => 32,
    [u8; 1] => 1,
    [u8; 2] => 2,
    [u8; 3] => 3,
    [u8; 4] => 4,
    [u8; 5] => 5,
    [u8; 6] => 6,
    [u8; 7] => 7,
    [u8; 8] => 8,
    [u8; 9] => 9,
    [u8; 10] => 10,
    [u8; 11] => 11,
    [u8; 12] => 12,
    [u8; 13] => 13,
    [u8; 14] => 14,
    [u8; 15] => 15,
    [u8; 16] => 16,
    [u8; 17] => 17,
    [u8; 18] => 18,
    [u8; 19] => 19,
    [u8; 20] => 20,
    [u8; 21] => 21,
    [u8; 22] => 22,
    [u8; 23] => 23,
    [u8; 24] => 24,
    [u8; 25] => 25,
    [u8; 26] => 26,
    [u8; 27] => 27,
    [u8; 28] => 28,
    [u8; 29] => 29,
    [u8; 30] => 30,
    [u8; 31] => 31,
    [u8; 32] => 32,
];

macro_rules! impl_sol_word_type_for_int {
    ($($t:ty => $size:expr),* $(,)?) => {
        $(impl SolWordType for $t {
            const PACKED_SIZE: usize = $size;
            type PackedBytes = FixedBytes<{Self::PACKED_SIZE}>;

            fn into_word(self) -> B256 {
                B256::left_padding_from(&SolValue::abi_encode(&self))
            }

            fn try_from_word(word: B256) -> Option<Self> {
                <Self as SolValue>::abi_decode(word.as_ref()).ok()
            }

            fn into_packed_word(self) -> Self::PackedBytes {
                FixedBytes::from_slice(&<Self as SolValue>::abi_encode_packed(&self))
            }

            fn try_from_packed_word(packed_word: Self::PackedBytes) -> Option<Self> {
                Some(<$t>::from_be_bytes(packed_word.0))
            }
        })?
    };
}

impl_sol_word_type_for_int![
    i8 => 1,
    i16 => 2,
    I24 => 3,
    i32 => 4,
    I40 => 5,
    I48 => 6,
    I56 => 7,
    i64 => 8,
    I72 => 9,
    I80 => 10,
    I88 => 11,
    I96 => 12,
    I104 => 13,
    I112 => 14,
    I120 => 15,
    i128 => 16,
    I136 => 17,
    I144 => 18,
    I152 => 19,
    I160 => 20,
    I168 => 21,
    I176 => 22,
    I184 => 23,
    I192 => 24,
    I200 => 25,
    I208 => 26,
    I216 => 27,
    I224 => 28,
    I232 => 29,
    I240 => 30,
    I248 => 31,
    I256 => 32,
];

macro_rules! impl_sol_word_type_for_word {
    ($($t:ty => $size:expr),* $(,)?) => {
        $(impl SolWordType for $t {
            const PACKED_SIZE: usize = $size;
            type PackedBytes = FixedBytes<{Self::PACKED_SIZE}>;

            fn into_word(self) -> B256 {
                B256::left_padding_from(&SolValue::abi_encode(&self))
            }

            fn try_from_word(word: B256) -> Option<Self> {
                <Self as SolValue>::abi_decode(word.as_ref()).ok()
            }

            fn into_packed_word(self) -> Self::PackedBytes {
                FixedBytes::from_slice(&<Self as SolValue>::abi_encode_packed(&self))
            }

            fn try_from_packed_word(packed_word: Self::PackedBytes) -> Option<Self> {
                let unpacked = B256::left_padding_from(packed_word.as_ref());
                Self::try_from_word(unpacked)
            }
        })?
    };
}

impl_sol_word_type_for_word![
    bool => 1,
    // u8 => 1,
    u16 => 2,
    U24 => 3,
    u32 => 4,
    U40 => 5,
    U48 => 6,
    U56 => 7,
    u64 => 8,
    U72 => 9,
    U80 => 10,
    U88 => 11,
    U96 => 12,
    U104 => 13,
    U112 => 14,
    U120 => 15,
    u128 => 16,
    U136 => 17,
    U144 => 18,
    U152 => 19,
    U160 => 20,
    U168 => 21,
    U176 => 22,
    U184 => 23,
    U192 => 24,
    U200 => 25,
    U208 => 26,
    U216 => 27,
    U224 => 28,
    U232 => 29,
    U240 => 30,
    U248 => 31,
    U256 => 32,
    Address => 20,
    Function => 24,
];

macro_rules! impl_sol_storage_type_value_for_word {
    ($($t:ty => $sol_t:ty),* $(,)?) => {
        $(
            impl SolStorageValue<$sol_t> for $t {
                fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
                where
                    Reader: StorageReader,
                {
                    let next = storage_reader.next_or_default::<<$t as SolWordType>::PackedBytes>();

                    if next.is_remaining_not_zero() {
                        return Err(SolLayoutError::remaining_bytes(next.remaining))
                    }

                    if next.children.is_empty() {
                        SolWordType::try_from_packed_word(next.word).ok_or_else(|| {
                            SolLayoutError::InvalidWordValue {
                                word: Bytes::copy_from_slice(next.word.as_ref()),
                            }
                        })
                    } else {
                        Err(SolLayoutError::UnexpectedChildren {
                            count: next.children.len(),
                        })
                    }
                }
            }
        )*
    };
}

impl_sol_storage_type_value_for_word![
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

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::*;

    use alloy_primitives::{address, b64, b128, b256, fixed_bytes, uint};

    fn test_encode_decode<T>(value: T, word: B256, packed_word: T::PackedBytes)
    where
        T: SolWordType + PartialEq + Debug + Copy,
        T::PackedBytes: PartialEq + Debug + Copy,
    {
        assert_eq!(value.into_word(), word, "into_word");
        assert_eq!(T::try_from_word(word), Some(value), "try_from_word");
        assert_eq!(value.into_packed_word(), packed_word, "into_packed_word");
        assert_eq!(
            T::try_from_packed_word(packed_word),
            Some(value),
            "try_from_packed_word"
        );
    }

    #[test]
    fn test_sol_word_fixed_bytes() {
        test_encode_decode(
            b128!("0x8f21dcf115a2dd360b97419e47da0246"),
            b256!("0x8f21dcf115a2dd360b97419e47da024600000000000000000000000000000000"),
            fixed_bytes!("0x8f21dcf115a2dd360b97419e47da0246"),
        );
        test_encode_decode(
            b64!("0xdf44796417b2e3ef"),
            b256!("0xdf44796417b2e3ef000000000000000000000000000000000000000000000000"),
            fixed_bytes!("0xdf44796417b2e3ef"),
        );
    }

    #[test]
    fn test_sol_word_address() {
        test_encode_decode(
            address!("0x2e48e3f2137c3cb1aef8254aa32dd06c26146735"),
            b256!("0x0000000000000000000000002e48e3f2137c3cb1aef8254aa32dd06c26146735"),
            fixed_bytes!("0x2e48e3f2137c3cb1aef8254aa32dd06c26146735"),
        );
    }

    #[test]
    fn test_sol_word_uint() {
        test_encode_decode(
            42u8,
            b256!("0x000000000000000000000000000000000000000000000000000000000000002a"),
            fixed_bytes!("0x2a"),
        );
        test_encode_decode(
            43u64,
            b256!("0x000000000000000000000000000000000000000000000000000000000000002b"),
            fixed_bytes!("0x000000000000002b"),
        );
        test_encode_decode(
            uint!(44_U168),
            b256!("0x000000000000000000000000000000000000000000000000000000000000002c"),
            fixed_bytes!("00000000000000000000000000000000000000002c"),
        );
    }

    #[test]
    fn test_sol_word_int() {
        test_encode_decode(
            42i128,
            b256!("0x000000000000000000000000000000000000000000000000000000000000002a"),
            fixed_bytes!("0x0000000000000000000000000000002a"),
        );
        test_encode_decode(
            -43i8,
            b256!("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd5"),
            fixed_bytes!("d5"),
        );
        test_encode_decode(
            -44i16,
            b256!("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd4"),
            fixed_bytes!("ffd4"),
        );
    }

    #[test]
    fn test_sol_word_bool() {
        test_encode_decode(
            true,
            b256!("0x0000000000000000000000000000000000000000000000000000000000000001"),
            fixed_bytes!("01"),
        );
        test_encode_decode(
            false,
            b256!("0x0000000000000000000000000000000000000000000000000000000000000000"),
            fixed_bytes!("00"),
        );
    }
}
