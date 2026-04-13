use std::convert::Infallible;

use sdecode_preimages_interface::PreimagesProviderMut;

use crate::{Image, Preimage, PreimageEntry, PreimagesProvider};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct EmptyPreimagesProvider;

impl EmptyPreimagesProvider {
    pub const fn new() -> Self {
        Self
    }
}

impl PreimagesProvider for EmptyPreimagesProvider {
    type Error = Infallible;

    fn nearest_lower_preimage(&self, _: &Image) -> Result<Option<PreimageEntry>, Self::Error> {
        Ok(None)
    }

    fn nearest_upper_preimage(&self, _: &Image) -> Result<Option<PreimageEntry>, Self::Error> {
        Ok(None)
    }

    fn exact_preimage(&self, _: &Image) -> Result<Option<Preimage>, Self::Error> {
        Ok(None)
    }

    fn is_empty(&self) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

impl PreimagesProviderMut for EmptyPreimagesProvider {
    type Error = Infallible;

    fn nearest_lower_preimage_mut(
        &mut self,
        _: &Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        Ok(None)
    }

    fn nearest_upper_preimage_mut(
        &mut self,
        _: &Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        Ok(None)
    }

    fn exact_preimage_mut(&mut self, _: &Image) -> Result<Option<Preimage>, Self::Error> {
        Ok(None)
    }

    fn is_empty_mut(&mut self) -> Result<bool, Self::Error> {
        Ok(true)
    }
}
