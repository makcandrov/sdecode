use ::alloy_primitives::{Address, FixedBytes, Function, aliases::*, keccak256};
use alloy_sol_types::SolValue;
use sdecode_core::{StorageReader, SubB256};

use crate::{data_types, utils::b256_to_u256};

use super::{SolLayoutError, SolStorageValue};

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
                        SolWordType::try_from_packed_word(next.word).ok_or(SolLayoutError::Err)
                    } else {
                        Err(SolLayoutError::Err)
                    }
                }
            }
        )*
    };
}

impl_sol_storage_type_value_for_word![
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
