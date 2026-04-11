use std::error::Error;

use crate::{Image, Preimage, PreimageEntry};

/// A boxed, type-erased [`PreimagesProvider`].
pub type BoxedPreimagesProvider<Error> = Box<dyn PreimagesProvider<Error = Error>>;

/// A boxed, type-erased [`PreimagesProviderMut`].
pub type BoxedPreimagesProviderMut<Error> = Box<dyn PreimagesProviderMut<Error = Error>>;

/// Read-only provider for looking up keccak256 preimages.
///
/// Implementors store a sorted set of preimage entries and support nearest-neighbor lookups by
/// image value.
#[auto_impl::auto_impl(&, &mut, Box, Rc, Arc)]
pub trait PreimagesProvider {
    /// The error type returned by provider operations.
    type Error: Error;

    /// Returns the entry with the largest image less than or equal to `image`, or `None` if no
    /// such entry exists.
    fn nearest_lower_preimage(&self, image: &Image) -> Result<Option<PreimageEntry>, Self::Error>;

    /// Returns the entry with the smallest image greater than or equal to `image`, or `None` if
    /// no such entry exists.
    fn nearest_upper_preimage(&self, image: &Image) -> Result<Option<PreimageEntry>, Self::Error>;

    /// Returns the preimage for an exact `image` match, or `None` if the image is not present.
    ///
    /// The default implementation delegates to [`nearest_lower_preimage`](Self::nearest_lower_preimage)
    /// and checks for an exact match.
    fn exact_preimage(&self, image: &Image) -> Result<Option<Preimage>, Self::Error> {
        if let Some(entry) = self.nearest_lower_preimage(image)? {
            Ok((entry.image_ref() == image).then_some(entry.into_preimage()))
        } else {
            Ok(None)
        }
    }

    /// Returns `true` if the provider contains no preimage entries.
    ///
    /// The default implementation delegates to
    /// [`nearest_upper_preimage`](Self::nearest_upper_preimage) starting from `Image::ZERO`.
    fn is_empty(&self) -> Result<bool, Self::Error> {
        Ok(self.nearest_upper_preimage(&Image::ZERO)?.is_none())
    }
}

/// Mutable provider for looking up keccak256 preimages.
///
/// Like [`PreimagesProvider`], but takes `&mut self`, allowing implementations to update internal
/// state (e.g. caches) on each lookup.
#[auto_impl::auto_impl(&mut, Box)]
pub trait PreimagesProviderMut {
    /// The error type returned by provider operations.
    type Error: Error;

    /// Returns the entry with the largest image less than or equal to `image`, or `None` if no
    /// such entry exists.
    fn nearest_lower_preimage_mut(
        &mut self,
        image: &Image,
    ) -> Result<Option<PreimageEntry>, Self::Error>;

    /// Returns the entry with the smallest image greater than or equal to `image`, or `None` if
    /// no such entry exists.
    fn nearest_upper_preimage_mut(
        &mut self,
        image: &Image,
    ) -> Result<Option<PreimageEntry>, Self::Error>;

    /// Returns the preimage for an exact `image` match, or `None` if the image is not present.
    ///
    /// The default implementation delegates to
    /// [`nearest_lower_preimage_mut`](Self::nearest_lower_preimage_mut) and checks for an exact
    /// match.
    fn exact_preimage_mut(&mut self, image: &Image) -> Result<Option<Preimage>, Self::Error> {
        if let Some(preimage) = self.nearest_lower_preimage_mut(image)? {
            Ok((preimage.image_ref() == image).then_some(preimage.into_preimage()))
        } else {
            Ok(None)
        }
    }

    /// Returns `true` if the provider contains no preimage entries.
    ///
    /// The default implementation delegates to
    /// [`nearest_upper_preimage_mut`](Self::nearest_upper_preimage_mut) starting from
    /// `Image::ZERO`.
    fn is_empty(&mut self) -> Result<bool, Self::Error> {
        Ok(self.nearest_upper_preimage_mut(&Image::ZERO)?.is_none())
    }
}

/// Wraps a [`PreimagesProvider`] to implement [`PreimagesProviderMut`].
///
/// This is useful when you have a read-only provider but need to pass it to an API that requires
/// [`PreimagesProviderMut`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WrapPreimagesProvider<P>(pub P);

impl<P> From<P> for WrapPreimagesProvider<P> {
    fn from(provider: P) -> Self {
        Self(provider)
    }
}

impl<P> WrapPreimagesProvider<P> {
    /// Creates a new wrapper around the given provider.
    pub const fn new(provider: P) -> Self {
        Self(provider)
    }
}

impl<P: PreimagesProvider> PreimagesProviderMut for WrapPreimagesProvider<P> {
    type Error = P::Error;

    #[inline]
    fn nearest_lower_preimage_mut(
        &mut self,
        image: &Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        self.0.nearest_lower_preimage(image)
    }

    #[inline]
    fn nearest_upper_preimage_mut(
        &mut self,
        image: &Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        self.0.nearest_upper_preimage(image)
    }

    #[inline]
    fn exact_preimage_mut(&mut self, image: &Image) -> Result<Option<Preimage>, Self::Error> {
        self.0.exact_preimage(image)
    }

    #[inline]
    fn is_empty(&mut self) -> Result<bool, Self::Error> {
        self.0.is_empty()
    }
}
