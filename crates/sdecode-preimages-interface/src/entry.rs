use alloy_primitives::{B256, Bytes, KECCAK256_EMPTY, U256, keccak256};
use quick_impl::quick_impl;

use crate::{Image, Preimage};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[quick_impl(impl Into, pub into_parts)]
pub struct PreimageEntry {
    #[quick_impl(pub const get = "{}_ref")]
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
    pub const fn image(&self) -> B256 {
        self.image
    }

    /// ```rust
    /// # use ::alloy_primitives::keccak256;
    /// # use ::sdecode_preimages_interface::PreimageEntry;
    /// const EMPTY: PreimageEntry = PreimageEntry::empty();
    /// assert_eq!(EMPTY.image(), keccak256(&[]));
    /// assert!(EMPTY.preimage().is_empty());
    /// ```
    #[inline]
    pub const fn empty() -> Self {
        Self {
            image: KECCAK256_EMPTY,
            preimage: Bytes::new(),
        }
    }

    #[inline]
    pub fn new(preimage: Preimage) -> Self {
        let image = keccak256(&preimage);
        Self::new_unchecked(image, preimage)
    }

    #[inline]
    pub fn new_unchecked(image: Image, preimage: Preimage) -> Self {
        debug_assert_eq!(image, keccak256(&preimage));
        Self { image, preimage }
    }

    #[inline]
    pub const fn image_u256(&self) -> U256 {
        U256::from_be_bytes(self.image().0)
    }
}
