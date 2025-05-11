use std::sync::atomic::{AtomicUsize, Ordering};

use overf::checked;
use quick_impl::quick_impl;

use crate::{Image, PreimageEntry, PreimagesProvider, PreimagesProviderMut, WrapPreimagesProvider};

/// A [`PreimagesProvider`] counting the underlying provider accesses.
#[derive(Debug)]
#[quick_impl]
pub struct CounterPreimagesProvider<P> {
    #[quick_impl(pub get = "{}", pub into)]
    provider: P,
    accesses: AtomicUsize,
}

/// A [`PreimagesProviderMut`] counting the underlying provider accesses.
#[derive(Debug)]
#[quick_impl]
pub struct CounterPreimagesProviderMut<P> {
    #[quick_impl(pub get = "{}", pub get_mut = "{}_mut", pub into)]
    provider: P,
    accesses: usize,
}

impl<P: PreimagesProvider> CounterPreimagesProviderMut<WrapPreimagesProvider<P>> {
    pub const fn new(preimages_provider: P) -> Self {
        Self::new_mut(WrapPreimagesProvider(preimages_provider))
    }
}

impl<P: PreimagesProviderMut> CounterPreimagesProviderMut<P> {
    pub const fn new_mut(preimages_provider: P) -> Self {
        Self {
            provider: preimages_provider,
            accesses: 0,
        }
    }
}

impl<P: PreimagesProvider> CounterPreimagesProvider<P> {
    pub const fn new(preimages_provider: P) -> Self {
        Self {
            provider: preimages_provider,
            accesses: AtomicUsize::new(0),
        }
    }
}

impl<P> CounterPreimagesProviderMut<P> {
    pub const fn accesses(&self) -> usize {
        self.accesses
    }
}

impl<P> CounterPreimagesProvider<P> {
    pub fn accesses(&self) -> usize {
        self.accesses.load(Ordering::Relaxed)
    }
}

impl<P: PreimagesProviderMut> PreimagesProviderMut for CounterPreimagesProviderMut<P> {
    type Error = P::Error;

    fn nearest_lower_preimage_mut(
        &mut self,
        image: Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        checked! { self.accesses += 1 };
        self.provider.nearest_lower_preimage_mut(image)
    }

    fn nearest_upper_preimage_mut(
        &mut self,
        image: Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        checked! { self.accesses += 1 };
        self.provider.nearest_upper_preimage_mut(image)
    }
}

impl<P: PreimagesProvider> PreimagesProvider for CounterPreimagesProvider<P> {
    type Error = P::Error;

    fn nearest_lower_preimage(&self, image: Image) -> Result<Option<PreimageEntry>, Self::Error> {
        self.accesses.fetch_add(1, Ordering::Relaxed);
        self.provider.nearest_lower_preimage(image)
    }

    fn nearest_upper_preimage(&self, image: Image) -> Result<Option<PreimageEntry>, Self::Error> {
        self.accesses.fetch_add(1, Ordering::Relaxed);
        self.provider.nearest_upper_preimage(image)
    }
}
