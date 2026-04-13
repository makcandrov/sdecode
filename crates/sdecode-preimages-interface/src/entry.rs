use std::borrow::Borrow;
use std::cmp::Ordering;

use alloy_primitives::{B256, Bytes, KECCAK256_EMPTY, U256, keccak256};

use crate::{Image, Preimage};

/// A keccak256 hash paired with its preimage bytes.
///
/// The entry guarantees (via [`debug_assert`]) that `image == keccak256(preimage)`.
/// Entries are ordered by their [`Image`] value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct PreimageEntry {
    /// The keccak256 hash of [`preimage`](Self::preimage).
    image: Image,

    /// The raw bytes that hash to [`image`](Self::image).
    preimage: Preimage,
}

impl PreimageEntry {
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
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            image: KECCAK256_EMPTY,
            preimage: Bytes::new(),
        }
    }

    /// Creates a new entry by computing the keccak256 hash of `preimage`.
    #[inline]
    #[must_use]
    pub fn new(preimage: Preimage) -> Self {
        let image = keccak256(&preimage);
        Self { image, preimage }
    }

    /// Creates a new entry from a pre-computed `image` and its `preimage`.
    ///
    /// In debug builds, asserts that `image == keccak256(preimage)`.
    #[inline]
    #[must_use]
    pub fn new_unchecked(image: Image, preimage: Preimage) -> Self {
        debug_assert_eq!(image, keccak256(&preimage));
        Self { image, preimage }
    }

    /// Returns the keccak256 hash by value.
    #[inline]
    #[must_use]
    pub const fn image(&self) -> B256 {
        self.image
    }

    /// Returns a reference to the keccak256 hash.
    #[inline]
    #[must_use]
    pub const fn image_ref(&self) -> &Image {
        &self.image
    }

    /// Returns a reference to the preimage bytes.
    #[inline]
    #[must_use]
    pub const fn preimage(&self) -> &Preimage {
        &self.preimage
    }

    /// Returns the image as a [`U256`] (big-endian interpretation).
    #[inline]
    #[must_use]
    pub const fn image_u256(&self) -> U256 {
        U256::from_be_bytes(self.image().0)
    }

    /// Returns the length of the preimage in bytes.
    #[inline]
    #[must_use]
    pub fn preimage_len(&self) -> usize {
        self.preimage.len()
    }

    /// Returns `true` if the preimage is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.preimage.is_empty()
    }

    /// Returns references to the image and preimage as a pair.
    #[inline]
    #[must_use]
    pub const fn as_parts(&self) -> (&Image, &Preimage) {
        (&self.image, &self.preimage)
    }

    /// Consumes the entry, returning the `(image, preimage)` pair.
    #[inline]
    #[must_use]
    pub fn into_parts(self) -> (Image, Preimage) {
        (self.image, self.preimage)
    }

    /// Consumes the entry, returning the preimage bytes.
    #[inline]
    #[must_use]
    pub fn into_preimage(self) -> Preimage {
        self.preimage
    }

    /// Returns a borrowed [`PreimageEntryRef`] view of this entry.
    #[inline]
    #[must_use]
    pub const fn as_ref(&self) -> PreimageEntryRef<'_> {
        PreimageEntryRef {
            image: &self.image,
            preimage: &self.preimage,
        }
    }
}

impl Default for PreimageEntry {
    fn default() -> Self {
        Self::empty()
    }
}

impl std::fmt::Display for PreimageEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} => {}", self.image, self.preimage)
    }
}

impl PartialOrd for PreimageEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PreimageEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.image.cmp(&other.image)
    }
}

impl Borrow<Image> for PreimageEntry {
    fn borrow(&self) -> &Image {
        self.image_ref()
    }
}

impl From<PreimageEntry> for (Image, Preimage) {
    fn from(entry: PreimageEntry) -> Self {
        entry.into_parts()
    }
}

impl<'a> From<&'a PreimageEntry> for (&'a Image, &'a Preimage) {
    fn from(entry: &'a PreimageEntry) -> Self {
        (entry.image_ref(), entry.preimage())
    }
}

impl<'a> From<&'a PreimageEntry> for (&'a [u8; 32], &'a [u8]) {
    fn from(entry: &'a PreimageEntry) -> Self {
        (&entry.image_ref().0, &entry.preimage().0)
    }
}

impl PartialEq<PreimageEntryRef<'_>> for PreimageEntry {
    fn eq(&self, other: &PreimageEntryRef<'_>) -> bool {
        self.image == *other.image_ref()
    }
}

impl PartialOrd<PreimageEntryRef<'_>> for PreimageEntry {
    fn partial_cmp(&self, other: &PreimageEntryRef<'_>) -> Option<Ordering> {
        Some(self.image.cmp(other.image_ref()))
    }
}

/// A borrowed view of a [`PreimageEntry`].
///
/// Holds references to the image and preimage, avoiding clones when only
/// read access is needed. Entries are ordered by their [`Image`] value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PreimageEntryRef<'a> {
    /// The keccak256 hash of [`preimage`](Self::preimage).
    image: &'a Image,

    /// The raw bytes that hash to [`image`](Self::image).
    preimage: &'a Preimage,
}

impl PreimageEntryRef<'static> {
    /// Creates an entry for the empty preimage (`keccak256(b"")`).
    ///
    /// ```rust
    /// # use ::alloy_primitives::keccak256;
    /// # use ::sdecode_preimages_interface::PreimageEntryRef;
    /// let empty = PreimageEntryRef::empty();
    /// assert_eq!(empty.image(), keccak256(&[]));
    /// assert!(empty.preimage().is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn empty() -> Self {
        static EMPTY_PREIMAGE: Bytes = Bytes::new();
        Self {
            image: &KECCAK256_EMPTY,
            preimage: &EMPTY_PREIMAGE,
        }
    }
}

impl<'a> PreimageEntryRef<'a> {
    /// Creates a new ref entry from a pre-computed `image` and its `preimage`.
    ///
    /// In debug builds, asserts that `image == keccak256(preimage)`.
    #[inline]
    #[must_use]
    pub fn new_unchecked(image: &'a Image, preimage: &'a Preimage) -> Self {
        debug_assert_eq!(*image, keccak256(preimage));
        PreimageEntryRef { image, preimage }
    }

    /// Returns the keccak256 hash by value.
    #[inline]
    #[must_use]
    pub const fn image(&self) -> B256 {
        *self.image
    }

    /// Returns a reference to the keccak256 hash.
    #[inline]
    #[must_use]
    pub const fn image_ref(&self) -> &Image {
        self.image
    }

    /// Returns a reference to the preimage bytes.
    #[inline]
    #[must_use]
    pub const fn preimage(&self) -> &Preimage {
        self.preimage
    }

    /// Returns the image as a [`U256`] (big-endian interpretation).
    #[inline]
    #[must_use]
    pub const fn image_u256(&self) -> U256 {
        U256::from_be_bytes(self.image().0)
    }

    /// Returns the length of the preimage in bytes.
    #[inline]
    #[must_use]
    pub fn preimage_len(&self) -> usize {
        self.preimage.len()
    }

    /// Returns `true` if the preimage is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.preimage.is_empty()
    }

    /// Returns references to the image and preimage as a pair.
    #[inline]
    #[must_use]
    pub const fn as_parts(&self) -> (&'a Image, &'a Preimage) {
        (self.image, self.preimage)
    }

    /// Converts this ref into an owned [`PreimageEntry`].
    #[inline]
    #[must_use]
    pub fn to_owned(&self) -> PreimageEntry {
        PreimageEntry::new_unchecked(*self.image, self.preimage.clone())
    }
}

impl Default for PreimageEntryRef<'static> {
    fn default() -> Self {
        Self::empty()
    }
}

impl std::fmt::Display for PreimageEntryRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} => {}", self.image, self.preimage)
    }
}

impl PartialOrd for PreimageEntryRef<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PreimageEntryRef<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.image.cmp(other.image)
    }
}

impl Borrow<Image> for PreimageEntryRef<'_> {
    fn borrow(&self) -> &Image {
        self.image
    }
}

impl<'a> From<PreimageEntryRef<'a>> for (&'a Image, &'a Preimage) {
    fn from(entry: PreimageEntryRef<'a>) -> Self {
        (entry.image, entry.preimage)
    }
}

impl<'a> From<PreimageEntryRef<'a>> for (&'a [u8; 32], &'a [u8]) {
    fn from(entry: PreimageEntryRef<'a>) -> Self {
        (&entry.image.0, &entry.preimage.0)
    }
}

impl<'a, 'b> From<&'b PreimageEntryRef<'a>> for (&'a Image, &'a Preimage) {
    fn from(entry: &'b PreimageEntryRef<'a>) -> Self {
        (entry.image, entry.preimage)
    }
}

impl<'a, 'b> From<&'b PreimageEntryRef<'a>> for (&'a [u8; 32], &'a [u8]) {
    fn from(entry: &'b PreimageEntryRef<'a>) -> Self {
        (&entry.image.0, &entry.preimage.0)
    }
}

impl PartialEq<PreimageEntry> for PreimageEntryRef<'_> {
    fn eq(&self, other: &PreimageEntry) -> bool {
        *self.image == other.image()
    }
}

impl PartialOrd<PreimageEntry> for PreimageEntryRef<'_> {
    fn partial_cmp(&self, other: &PreimageEntry) -> Option<Ordering> {
        Some(self.image.cmp(other.image_ref()))
    }
}

impl<'a> From<&'a PreimageEntry> for PreimageEntryRef<'a> {
    fn from(entry: &'a PreimageEntry) -> Self {
        entry.as_ref()
    }
}

impl From<PreimageEntryRef<'_>> for PreimageEntry {
    fn from(entry: PreimageEntryRef<'_>) -> Self {
        Self::new_unchecked(*entry.image, entry.preimage.clone())
    }
}
