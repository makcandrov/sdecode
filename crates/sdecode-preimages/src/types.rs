use alloy_primitives::{B256, Bytes, KECCAK256_EMPTY, U256, keccak256};
use quick_impl::quick_impl;

use crate::utils::b256_to_u256;

pub type Image = B256;
pub type Preimage = Bytes;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[quick_impl(impl Into, pub into_parts)]
pub struct PreimageEntry {
    #[quick_impl(pub const get = "{}_ref", pub get_clone = "{}")]
    image: Image,

    #[quick_impl(pub const get = "{}", pub into)]
    preimage: Preimage,
}

impl Default for PreimageEntry {
    fn default() -> Self {
        Self::empty()
    }
}

impl PartialOrd for PreimageEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PreimageEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.image.cmp(&other.image)
    }
}

impl PreimageEntry {
    /// ```rust
    /// # use ::alloy_primitives::keccak256;
    /// # use ::sdecode_preimages::PreimageEntry;
    /// let empty = PreimageEntry::empty();
    ///
    /// assert_eq!(empty.image(), keccak256(&[]));
    /// assert!(empty.preimage().is_empty());
    /// ```
    pub const fn empty() -> Self {
        Self {
            image: KECCAK256_EMPTY,
            preimage: Bytes::new(),
        }
    }

    pub fn new(preimage: Preimage) -> Self {
        let image = keccak256(&preimage);
        Self::new_unchecked(image, preimage)
    }

    #[inline(always)]
    pub fn new_unchecked(image: Image, preimage: Preimage) -> Self {
        debug_assert_eq!(image, keccak256(&preimage));
        Self { image, preimage }
    }

    #[inline(always)]
    pub const fn image_u256(&self) -> U256 {
        b256_to_u256(*self.image_ref())
    }
}
