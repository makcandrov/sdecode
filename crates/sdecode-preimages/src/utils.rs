use alloy_primitives::{B256, U256};

#[inline(always)]
pub fn b256_to_u256(value: B256) -> U256 {
    From::from(value)
}
