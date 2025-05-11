use alloy_primitives::{B256, U256};

#[inline(always)]
pub fn b256_to_u256(value: B256) -> U256 {
    From::from(value)
}

#[inline(always)]
pub fn slice_is_zero(slice: impl AsRef<[u8]>) -> bool {
    slice.as_ref().iter().all(|b| *b == 0)
}
