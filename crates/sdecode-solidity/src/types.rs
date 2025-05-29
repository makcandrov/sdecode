use std::marker::PhantomData;

use alloy_sol_types::SolType;
pub use alloy_sol_types::sol_data::{
    Address, Array, Bool, ByteCount, Bytes, FixedArray, FixedBytes, Function, Int, IntBitCount,
    String, SupportedFixedBytes, SupportedInt, Uint,
};

pub trait SolStorageType {
    const SOL_STORAGE_NAME: &'static str;
}
pub trait SolMappingKeyType: SolStorageType {}

pub struct Mapping<K, V>(PhantomData<(K, V)>);

impl SolStorageType for Bool {
    const SOL_STORAGE_NAME: &'static str = <Self as SolType>::SOL_NAME;
}
impl SolMappingKeyType for Bool {}

impl SolStorageType for Address {
    const SOL_STORAGE_NAME: &'static str = <Self as SolType>::SOL_NAME;
}
impl SolMappingKeyType for Address {}

impl SolStorageType for Function {
    const SOL_STORAGE_NAME: &'static str = <Self as SolType>::SOL_NAME;
}
impl SolMappingKeyType for Function {}

impl SolStorageType for Bytes {
    const SOL_STORAGE_NAME: &'static str = <Self as SolType>::SOL_NAME;
}
impl SolMappingKeyType for Bytes {}

impl SolStorageType for String {
    const SOL_STORAGE_NAME: &'static str = <Self as SolType>::SOL_NAME;
}
impl SolMappingKeyType for String {}

impl<const N: usize> SolStorageType for FixedBytes<N>
where
    ByteCount<N>: SupportedFixedBytes,
{
    const SOL_STORAGE_NAME: &'static str = <Self as SolType>::SOL_NAME;
}
impl<const N: usize> SolMappingKeyType for FixedBytes<N> where ByteCount<N>: SupportedFixedBytes {}

impl<const N: usize> SolStorageType for Uint<N>
where
    IntBitCount<N>: SupportedInt,
{
    const SOL_STORAGE_NAME: &'static str = <Self as SolType>::SOL_NAME;
}
impl<const N: usize> SolMappingKeyType for Uint<N> where IntBitCount<N>: SupportedInt {}

impl<const N: usize> SolStorageType for Int<N>
where
    IntBitCount<N>: SupportedInt,
{
    const SOL_STORAGE_NAME: &'static str = <Self as SolType>::SOL_NAME;
}
impl<const N: usize> SolMappingKeyType for Int<N> where IntBitCount<N>: SupportedInt {}

impl<T: SolStorageType, const N: usize> SolStorageType for FixedArray<T, N> {
    const SOL_STORAGE_NAME: &'static str = NameBuffer::new()
        .write_str(T::SOL_STORAGE_NAME)
        .write_byte(b'[')
        .write_usize(N)
        .write_byte(b']')
        .as_str();
}

impl<T: SolStorageType> SolStorageType for Array<T> {
    const SOL_STORAGE_NAME: &'static str = NameBuffer::new()
        .write_str(T::SOL_STORAGE_NAME)
        .write_str("[]")
        .as_str();
}

impl<K: SolMappingKeyType, V: SolStorageType> SolStorageType for Mapping<K, V> {
    const SOL_STORAGE_NAME: &'static str = NameBuffer::new()
        .write_str("mapping(")
        .write_str(K::SOL_STORAGE_NAME)
        .write_str(" => ")
        .write_str(V::SOL_STORAGE_NAME)
        .write_byte(b')')
        .as_str();
}

const NAME_CAP: usize = 256;

/// Simple buffer for constructing strings at compile time.
///
/// Copied from `alloy_sol_types`.
#[must_use]
struct NameBuffer {
    buffer: [u8; NAME_CAP],
    len: usize,
}

impl NameBuffer {
    const fn new() -> Self {
        Self {
            buffer: [0; NAME_CAP],
            len: 0,
        }
    }

    const fn write_str(self, s: &str) -> Self {
        self.write_bytes(s.as_bytes())
    }

    const fn write_bytes(mut self, s: &[u8]) -> Self {
        let mut i = 0;
        while i < s.len() {
            self.buffer[self.len + i] = s[i];
            i += 1;
        }
        self.len += s.len();
        self
    }

    const fn write_byte(mut self, b: u8) -> Self {
        self.buffer[self.len] = b;
        self.len += 1;
        self
    }

    const fn write_usize(mut self, number: usize) -> Self {
        let Some(digits) = number.checked_ilog10() else {
            return self.write_byte(b'0');
        };
        let digits = digits as usize + 1;

        let mut n = number;
        let mut i = self.len + digits;
        while n > 0 {
            i -= 1;
            self.buffer[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }
        self.len += digits;

        self
    }

    // const fn pop(mut self) -> Self {
    //     self.len -= 1;
    //     self
    // }

    const fn as_bytes(&self) -> &[u8] {
        assert!(self.len <= self.buffer.len());
        unsafe { core::slice::from_raw_parts(self.buffer.as_ptr(), self.len) }
    }

    const fn as_str(&self) -> &str {
        match core::str::from_utf8(self.as_bytes()) {
            Ok(s) => s,
            Err(_) => panic!("wrote invalid UTF-8"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapping_name() {
        assert_eq!(
            <Mapping<Bool, Uint<136>> as SolStorageType>::SOL_STORAGE_NAME,
            "mapping(bool => uint136)"
        );
    }
}
