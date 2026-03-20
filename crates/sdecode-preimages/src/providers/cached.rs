use quick_impl::quick_impl;

use crate::{Image, PreimageEntry, PreimagesProvider, PreimagesProviderMut, WrapPreimagesProvider};

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
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[quick_impl(pub into_parts)]
#[quick_impl_all(pub const get = "{}", pub const get_mut = "{}_mut", pub into, pub take, pub replace)]
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
