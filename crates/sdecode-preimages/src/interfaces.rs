use std::error::Error;

use quick_impl::quick_impl;

use crate::{Image, Preimage, PreimageEntry};

pub type BoxedPreimagesProvider<Error> = Box<dyn PreimagesProvider<Error = Error>>;
pub type BoxedPreimagesProviderMut<Error> = Box<dyn PreimagesProviderMut<Error = Error>>;

#[auto_impl::auto_impl(&, &mut, Box, Rc, Arc)]
pub trait PreimagesProvider {
    type Error: Error;

    /// Nearest lower preimage.
    fn nearest_lower_preimage(&self, image: Image) -> Result<Option<PreimageEntry>, Self::Error>;

    /// Nearest upper preimage.
    fn nearest_upper_preimage(&self, image: Image) -> Result<Option<PreimageEntry>, Self::Error>;

    /// Exact preimage.
    fn exact_preimage(&self, image: Image) -> Result<Option<Preimage>, Self::Error> {
        if let Some(entry) = self.nearest_lower_preimage(image)? {
            Ok((entry.image() == image).then_some(entry.into_preimage()))
        } else {
            Ok(None)
        }
    }
}

#[auto_impl::auto_impl(&mut, Box)]
pub trait PreimagesProviderMut {
    type Error: Error;

    /// Nearest lower preimage.
    fn nearest_lower_preimage_mut(
        &mut self,
        image: Image,
    ) -> Result<Option<PreimageEntry>, Self::Error>;

    /// Nearest upper preimage.
    fn nearest_upper_preimage_mut(
        &mut self,
        image: Image,
    ) -> Result<Option<PreimageEntry>, Self::Error>;

    /// Exact preimage.
    fn exact_preimage_mut(&mut self, image: Image) -> Result<Option<Preimage>, Self::Error> {
        if let Some(preimage) = self.nearest_lower_preimage_mut(image)? {
            Ok((preimage.image() == image).then_some(preimage.into_preimage()))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[quick_impl]
pub struct WrapPreimagesProvider<P>(#[quick_impl(impl From, impl Deref, impl DerefMut)] pub P);

impl<P> WrapPreimagesProvider<P> {
    pub const fn new(provider: P) -> Self {
        Self(provider)
    }
}

impl<P: PreimagesProvider> PreimagesProviderMut for WrapPreimagesProvider<P> {
    type Error = P::Error;

    #[inline(always)]
    fn nearest_lower_preimage_mut(
        &mut self,
        image: Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        self.0.nearest_lower_preimage(image)
    }

    #[inline(always)]
    fn nearest_upper_preimage_mut(
        &mut self,
        image: Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        self.0.nearest_upper_preimage(image)
    }
}

fn _assert_dyn_compatible<E: Error>(
    _: &dyn PreimagesProvider<Error = E>,
    _: &dyn PreimagesProviderMut<Error = E>,
) {
}
