use std::borrow::Borrow;

use quick_impl::quick_impl;
use sdecode_preimages_interface::{PreimagesWriter, PreimagesWriterMut};

use crate::{Image, PreimageEntry, PreimagesProvider, PreimagesProviderMut, WrapPreimagesProvider};

pub trait PreimagesCache<P: PreimagesProviderMut>: Sized {
    fn nearest_lower_preimage_mut(
        &mut self,
        provider: &mut P,
        image: &Image,
    ) -> Result<Option<PreimageEntry>, P::Error>;

    fn nearest_upper_preimage_mut(
        &mut self,
        provider: &mut P,
        image: &Image,
    ) -> Result<Option<PreimageEntry>, P::Error>;
}

pub trait PreimagesCacheInit<P: PreimagesProviderMut>: PreimagesCache<P> {
    type Params;
    type InitError;

    fn new_init(provider: &mut P, params: Self::Params) -> Result<Self, Self::InitError>;
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
    pub const fn from_raw(provider: P, cache: C) -> Self {
        Self::from_raw_mut(WrapPreimagesProvider(provider), cache)
    }
}

impl<P, C> CachedProvider<WrapPreimagesProvider<P>, C>
where
    P: PreimagesProvider,
    C: PreimagesCacheInit<WrapPreimagesProvider<P>>,
{
    #[inline]
    pub fn new(provider: P, params: C::Params) -> Result<Self, C::InitError> {
        Self::new_mut(WrapPreimagesProvider(provider), params)
    }
}

impl<P, C> CachedProvider<P, C>
where
    P: PreimagesProviderMut,
    C: PreimagesCache<P>,
{
    #[inline]
    pub const fn from_raw_mut(provider: P, cache: C) -> Self {
        Self { provider, cache }
    }
}

impl<P, C> CachedProvider<P, C>
where
    P: PreimagesProviderMut,
    C: PreimagesCacheInit<P>,
{
    #[inline]
    pub fn new_mut(mut provider: P, params: C::Params) -> Result<Self, C::InitError> {
        let cache = C::new_init(&mut provider, params)?;
        Ok(Self::from_raw_mut(provider, cache))
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
        image: &Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        PreimagesCache::nearest_lower_preimage_mut(&mut self.cache, &mut self.provider, image)
    }

    fn nearest_upper_preimage_mut(
        &mut self,
        image: &Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        PreimagesCache::nearest_upper_preimage_mut(&mut self.cache, &mut self.provider, image)
    }
}

impl<P, C> PreimagesWriter for CachedProvider<P, C>
where
    C: PreimagesWriter,
{
    type Error = C::Error;

    #[inline]
    fn write_preimages<'a>(
        &self,
        preimages: impl IntoIterator<Item = impl Borrow<PreimageEntry>>,
    ) -> Result<(), Self::Error> {
        self.cache.write_preimages(preimages)
    }

    #[inline]
    fn write_preimage_entry(&self, preimage: &PreimageEntry) -> Result<(), Self::Error> {
        self.cache.write_preimage_entry(preimage)
    }
}

impl<P, C> PreimagesWriterMut for CachedProvider<P, C>
where
    C: PreimagesWriterMut,
{
    type Error = C::Error;

    #[inline]
    fn write_preimage_entry_mut(&mut self, preimage: &PreimageEntry) -> Result<(), Self::Error> {
        self.cache.write_preimage_entry_mut(preimage)
    }

    #[inline]
    fn write_preimages_mut<'a>(
        &mut self,
        preimages: impl IntoIterator<Item = impl Borrow<PreimageEntry>>,
    ) -> Result<(), Self::Error> {
        self.cache.write_preimages_mut(preimages)
    }
}
