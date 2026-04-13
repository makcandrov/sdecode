use std::sync::atomic::{AtomicUsize, Ordering};

use quick_impl::quick_impl;

use crate::{Image, PreimageEntry, PreimagesProvider, PreimagesProviderMut};

/// A wrapper that counts every lookup forwarded to the inner provider.
///
/// Each call to [`nearest_lower_preimage`](PreimagesProvider::nearest_lower_preimage),
/// [`nearest_upper_preimage`](PreimagesProvider::nearest_upper_preimage), and their `_mut`
/// counterparts increments an internal counter. Other trait methods (such as
/// [`exact_preimage`](PreimagesProvider::exact_preimage)) are not counted unless their default
/// implementation delegates to one of the counted methods.
///
/// The counter uses an [`AtomicUsize`] so it can be read through a shared reference, which is
/// convenient when the underlying provider is itself shared.
///
/// # Example
///
/// ```rust
/// use alloy_primitives::{B256, Bytes};
/// use sdecode_preimages::{
///     InMemoryPreimages, PreimagesProvider, misc::CountingPreimagesProvider,
/// };
///
/// let mut db = InMemoryPreimages::new();
/// db.insert(Bytes::from_static(b"hello"));
///
/// let counter = CountingPreimagesProvider::new_mut(&db);
/// assert_eq!(counter.accesses(), 0);
///
/// let _ = counter.nearest_lower_preimage(&B256::ZERO).unwrap();
/// let _ = counter.nearest_upper_preimage(&B256::ZERO).unwrap();
/// assert_eq!(counter.accesses(), 2);
///
/// counter.reset();
/// assert_eq!(counter.accesses(), 0);
/// ```
#[derive(Debug)]
#[quick_impl]
pub struct CountingPreimagesProvider<P> {
    #[quick_impl(pub get = "{}", pub get_mut = "{}_mut", pub into)]
    provider: P,
    accesses: AtomicUsize,
}

impl<P> CountingPreimagesProvider<P> {
    /// Wraps the given provider with a counter initialized to zero.
    #[inline]
    pub const fn new(provider: P) -> Self {
        Self {
            provider,
            accesses: AtomicUsize::new(0),
        }
    }
}

impl<P> CountingPreimagesProvider<P> {
    /// Returns the number of lookups forwarded to the inner provider so far.
    #[inline]
    pub fn accesses(&self) -> usize {
        self.accesses.load(Ordering::Relaxed)
    }

    /// Resets the access counter to zero.
    #[inline]
    pub fn reset(&self) {
        self.accesses.store(0, Ordering::Relaxed);
    }

    #[inline]
    fn record_access(&self) {
        self.accesses.fetch_add(1, Ordering::Relaxed);
    }
}

impl<P: PreimagesProvider> PreimagesProvider for CountingPreimagesProvider<P> {
    type Error = P::Error;

    fn nearest_lower_preimage(&self, image: &Image) -> Result<Option<PreimageEntry>, Self::Error> {
        self.record_access();
        self.provider.nearest_lower_preimage(image)
    }

    fn nearest_upper_preimage(&self, image: &Image) -> Result<Option<PreimageEntry>, Self::Error> {
        self.record_access();
        self.provider.nearest_upper_preimage(image)
    }
}

impl<P: PreimagesProviderMut> PreimagesProviderMut for CountingPreimagesProvider<P> {
    type Error = P::Error;

    fn nearest_lower_preimage_mut(
        &mut self,
        image: &Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        self.record_access();
        self.provider.nearest_lower_preimage_mut(image)
    }

    fn nearest_upper_preimage_mut(
        &mut self,
        image: &Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        self.record_access();
        self.provider.nearest_upper_preimage_mut(image)
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{B256, Bytes};

    use super::*;
    use crate::InMemoryPreimages;

    fn fixture() -> InMemoryPreimages {
        let mut db = InMemoryPreimages::new();
        db.insert(Bytes::from_static(b"hello"));
        db.insert(Bytes::from_static(b"world"));
        db
    }

    #[test]
    fn counts_shared_lookups() {
        let counter = CountingPreimagesProvider::new(fixture());
        assert_eq!(counter.accesses(), 0);

        let _ = counter.nearest_lower_preimage(&B256::ZERO).unwrap();
        let _ = counter.nearest_upper_preimage(&B256::ZERO).unwrap();
        assert_eq!(counter.accesses(), 2);
    }

    #[test]
    fn counts_mut_lookups() {
        let mut counter = CountingPreimagesProvider::new(fixture());

        let _ = counter.nearest_lower_preimage_mut(&B256::ZERO).unwrap();
        let _ = counter.nearest_upper_preimage_mut(&B256::ZERO).unwrap();
        assert_eq!(counter.accesses(), 2);
    }

    #[test]
    fn reset_clears_count() {
        let counter = CountingPreimagesProvider::new(fixture());
        let _ = counter.nearest_lower_preimage(&B256::ZERO).unwrap();
        assert_eq!(counter.accesses(), 1);

        counter.reset();
        assert_eq!(counter.accesses(), 0);

        let _ = counter.nearest_upper_preimage(&B256::ZERO).unwrap();
        assert_eq!(counter.accesses(), 1);
    }

    #[test]
    fn into_provider_returns_inner() {
        let db = fixture();
        let counter = CountingPreimagesProvider::new(db.clone());
        let _ = counter.nearest_lower_preimage(&B256::ZERO).unwrap();
        let inner = counter.into_provider();
        assert_eq!(inner, db);
    }
}
