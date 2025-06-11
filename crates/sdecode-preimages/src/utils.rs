use alloy_primitives::{B256, U256, b256};

pub const B256_MAX: B256 =
    b256!("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");

#[inline(always)]
pub fn b256_to_u256(value: B256) -> U256 {
    From::from(value)
}
