use alloy_primitives::{B256, U256};

#[inline(always)]
pub const fn b256_to_u256(value: B256) -> U256 {
    U256::from_be_bytes(value.0)
}

#[inline(always)]
pub fn slice_is_zero(slice: impl AsRef<[u8]>) -> bool {
    slice.as_ref().iter().all(|b| *b == 0)
}
