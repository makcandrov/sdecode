use std::convert::Infallible;

use crate::{Image, Preimage, PreimageEntry, PreimagesProvider};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EmptyPreimagesProvider;

impl EmptyPreimagesProvider {
    pub const fn new() -> Self {
        Self
    }
}

impl PreimagesProvider for EmptyPreimagesProvider {
    type Error = Infallible;

    fn nearest_lower_preimage(&self, _image: Image) -> Result<Option<PreimageEntry>, Self::Error> {
        Ok(None)
    }

    fn nearest_upper_preimage(&self, _image: Image) -> Result<Option<PreimageEntry>, Self::Error> {
        Ok(None)
    }

    fn exact_preimage(&self, _image: Image) -> Result<Option<Preimage>, Self::Error> {
        Ok(None)
    }
}
