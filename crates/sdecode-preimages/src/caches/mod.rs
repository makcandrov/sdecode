use crate::{Image, PreimageEntry, PreimagesProvider, PreimagesProviderMut, WrapPreimagesProvider};

mod approx;
pub use approx::ApproxCache;

mod general;
pub use general::GeneralPreimagesCache;

mod storage;
pub use storage::StoragePreimagesCache;

pub trait PreimagesCache<P: PreimagesProviderMut>: Sized {
    fn new(provider: &mut P) -> Result<Self, P::Error>;

    fn nearest_lower_preimage_mut(
        &mut self,
        provider: &mut P,
        image: Image,
    ) -> Result<Option<PreimageEntry>, P::Error>;

    fn nearest_upper_preimage_mut(
        &mut self,
        provider: &mut P,
        image: Image,
    ) -> Result<Option<PreimageEntry>, P::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CachedProvider<P, C> {
    provider: P,
    cache: C,
}

impl<P, C> CachedProvider<WrapPreimagesProvider<P>, C>
where
    P: PreimagesProvider,
    C: PreimagesCache<WrapPreimagesProvider<P>>,
{
    #[inline]
    pub const fn new(provider: P, cache: C) -> Self {
        Self::new_mut(WrapPreimagesProvider(provider), cache)
    }
}

impl<P, C> CachedProvider<P, C>
where
    P: PreimagesProviderMut,
    C: PreimagesCache<P>,
{
    #[inline]
    pub const fn new_mut(provider: P, cache: C) -> Self {
        Self { provider, cache }
    }
}

impl<P, C> PreimagesProviderMut for CachedProvider<P, C>
where
    P: PreimagesProviderMut,
    C: PreimagesCache<P>,
{
    type Error = P::Error;

    fn nearest_lower_preimage_mut(
        &mut self,
        image: Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        PreimagesCache::nearest_lower_preimage_mut(&mut self.cache, &mut self.provider, image)
    }

    fn nearest_upper_preimage_mut(
        &mut self,
        image: Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        PreimagesCache::nearest_upper_preimage_mut(&mut self.cache, &mut self.provider, image)
    }
}
