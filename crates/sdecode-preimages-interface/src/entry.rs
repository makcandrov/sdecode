use alloy_primitives::{B256, Bytes, KECCAK256_EMPTY, U256, keccak256};
use quick_impl::quick_impl;

use crate::{Image, Preimage};

/// A keccak256 hash paired with its preimage bytes.
///
/// The entry guarantees (via [`debug_assert`]) that `image == keccak256(preimage)`.
/// Entries are ordered by their [`Image`] value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[quick_impl(impl Into, pub into_parts)]
pub struct PreimageEntry {
    /// The keccak256 hash of [`preimage`](Self::preimage).
    #[quick_impl(pub const get = "{}_ref")]
    image: Image,

    /// The raw bytes that hash to [`image`](Self::image).
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
    /// Returns the keccak256 hash.
    pub const fn image(&self) -> B256 {
        self.image
    }

    /// Creates an entry for the empty preimage (`keccak256(b"")`).
    ///
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

    /// Creates a new entry by computing the keccak256 hash of `preimage`.
    #[inline]
    pub fn new(preimage: Preimage) -> Self {
        let image = keccak256(&preimage);
        Self::new_unchecked(image, preimage)
    }

    /// Creates a new entry from a pre-computed `image` and its `preimage`.
    ///
    /// In debug builds, asserts that `image == keccak256(preimage)`.
    #[inline]
    pub fn new_unchecked(image: Image, preimage: Preimage) -> Self {
        debug_assert_eq!(image, keccak256(&preimage));
        Self { image, preimage }
    }

    /// Returns the image as a [`U256`] (big-endian interpretation).
    #[inline]
    pub const fn image_u256(&self) -> U256 {
        U256::from_be_bytes(self.image().0)
    }
}
