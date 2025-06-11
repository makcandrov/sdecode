use std::{
    collections::{BTreeMap, btree_map},
    convert::Infallible,
};

use alloy_primitives::keccak256;
use quick_impl::quick_impl;

use crate::{Image, Preimage, PreimageEntry, PreimagesProvider};

/// Preimages database in memory only.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[quick_impl(impl From, impl Into)]
pub struct MemoryPreimagesProvider {
    #[cfg_attr(feature = "serde", serde(flatten))]
    preimages: BTreeMap<Image, Preimage>,
}

impl MemoryPreimagesProvider {
    pub const fn new() -> Self {
        Self {
            preimages: BTreeMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.preimages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.preimages.is_empty()
    }

    /// New structure filled with `n` random preimages.
    pub fn random_filled(n: usize) -> Self {
        let mut res = Self::default();
        for _ in 0..n {
            res.insert(Preimage::copy_from_slice(&Image::random().0));
        }
        res
    }

    /// Insert a preimage.
    pub fn insert(&mut self, preimage: Preimage) -> Image {
        let image = keccak256(&preimage);
        self.insert_unchecked(image, preimage);
        image
    }

    /// Insert a preimage entry.
    pub fn insert_entry(&mut self, entry: PreimageEntry) -> bool {
        let (image, preimage) = entry.into_parts();
        self.insert_unchecked(image, preimage)
    }

    /// Insert a preimage without checking the validity.
    pub fn insert_unchecked(&mut self, image: Image, preimage: Preimage) -> bool {
        self.preimages.insert(image, preimage).is_none()
    }

    pub fn insert_unchecked_with(
        &mut self,
        image: Image,
        mut preimage: impl FnMut() -> Preimage,
    ) -> bool {
        match self.preimages.entry(image) {
            btree_map::Entry::Occupied(_) => false,
            btree_map::Entry::Vacant(e) => {
                e.insert(preimage());
                true
            }
        }
    }

    pub fn from_iter_unchecked(iter: impl IntoIterator<Item = (Image, Preimage)>) -> Self {
        iter.into_iter()
            .map(|(image, preimage)| PreimageEntry::new_unchecked(image, preimage))
            .collect()
    }
}

impl FromIterator<PreimageEntry> for MemoryPreimagesProvider {
    fn from_iter<T: IntoIterator<Item = PreimageEntry>>(iter: T) -> Self {
        Self {
            preimages: iter.into_iter().map(PreimageEntry::into_parts).collect(),
        }
    }
}

impl<'a> FromIterator<&'a PreimageEntry> for MemoryPreimagesProvider {
    fn from_iter<T: IntoIterator<Item = &'a PreimageEntry>>(iter: T) -> Self {
        iter.into_iter().cloned().collect()
    }
}

impl PreimagesProvider for MemoryPreimagesProvider {
    type Error = Infallible;

    fn nearest_lower_preimage(&self, image: Image) -> Result<Option<PreimageEntry>, Self::Error> {
        Ok(self
            .preimages
            .range(..=image)
            .max()
            .map(|(image, preimage)| PreimageEntry::new_unchecked(*image, preimage.clone())))
    }

    fn nearest_upper_preimage(&self, image: Image) -> Result<Option<PreimageEntry>, Self::Error> {
        Ok(self
            .preimages
            .range(image..)
            .min()
            .map(|(image, preimage)| PreimageEntry::new_unchecked(*image, preimage.clone())))
    }
}

impl IntoIterator for MemoryPreimagesProvider {
    type Item = PreimageEntry;

    type IntoIter = std::iter::Map<
        btree_map::IntoIter<Image, Preimage>,
        fn((Image, Preimage)) -> PreimageEntry,
    >;

    fn into_iter(self) -> Self::IntoIter {
        #[inline(always)]
        fn map((image, preimage): (Image, Preimage)) -> PreimageEntry {
            PreimageEntry::new_unchecked(image, preimage)
        }
        self.preimages.into_iter().map(map)
    }
}
