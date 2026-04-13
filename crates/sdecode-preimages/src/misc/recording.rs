use quick_impl::quick_impl;

use crate::{
    Image, InMemoryPreimages, PreimageEntry, PreimagesProvider, PreimagesProviderMut,
    WrapPreimagesProvider,
};

/// A wrapper that records every entry returned by the inner provider.
///
/// On each successful lookup the returned entry is inserted into an [`InMemoryPreimages`] store,
/// accessible via [`recorded`](Self::recorded). This lets you materialize a lazy or remote
/// provider into a local snapshot that can later be persisted or replayed offline.
///
/// Only entries actually returned by the inner provider are recorded — queries that resolve to
/// `None` produce no recording.
///
/// # Example
///
/// ```rust
/// use alloy_primitives::Bytes;
/// use sdecode_preimages::{
///     InMemoryPreimages, PreimagesProviderMut, misc::RecordingPreimagesProvider,
/// };
///
/// let mut db = InMemoryPreimages::new();
/// let img = db.insert(Bytes::from_static(b"hello"));
///
/// let mut recorder = RecordingPreimagesProvider::new(&db);
/// let _ = recorder.nearest_lower_preimage_mut(&img).unwrap();
///
/// let recorded = recorder.into_recorded();
/// assert_eq!(recorded.len(), 1);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[quick_impl]
pub struct RecordingPreimagesProvider<P> {
    #[quick_impl(pub get = "{}", pub get_mut = "{}_mut", pub into)]
    provider: P,

    #[quick_impl(pub get = "{}", pub into)]
    recorded: InMemoryPreimages,
}

impl<P: PreimagesProvider> RecordingPreimagesProvider<WrapPreimagesProvider<P>> {
    /// Wraps a [`PreimagesProvider`], adapting it to [`PreimagesProviderMut`] in the process.
    #[inline]
    pub const fn new(provider: P) -> Self {
        Self::new_mut(WrapPreimagesProvider(provider))
    }
}

impl<P: PreimagesProviderMut> RecordingPreimagesProvider<P> {
    /// Wraps a [`PreimagesProviderMut`].
    #[inline]
    pub const fn new_mut(provider: P) -> Self {
        Self {
            provider,
            recorded: InMemoryPreimages::new(),
        }
    }
}

impl<P> RecordingPreimagesProvider<P> {
    /// Removes and returns the recorded entries, leaving an empty store behind.
    #[inline]
    pub fn take_recorded(&mut self) -> InMemoryPreimages {
        std::mem::take(&mut self.recorded)
    }
}

impl<P: PreimagesProviderMut> PreimagesProviderMut for RecordingPreimagesProvider<P> {
    type Error = P::Error;

    fn nearest_lower_preimage_mut(
        &mut self,
        image: &Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        let entry = self.provider.nearest_lower_preimage_mut(image)?;
        if let Some(entry) = &entry {
            self.recorded
                .insert_unchecked_with(entry.image(), || entry.preimage().clone());
        }
        Ok(entry)
    }

    fn nearest_upper_preimage_mut(
        &mut self,
        image: &Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        let entry = self.provider.nearest_upper_preimage_mut(image)?;
        if let Some(entry) = &entry {
            self.recorded
                .insert_unchecked_with(entry.image(), || entry.preimage().clone());
        }
        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{B256, Bytes};

    use super::*;
    use crate::EmptyPreimagesProvider;

    fn fixture() -> (InMemoryPreimages, B256, B256) {
        let mut db = InMemoryPreimages::new();
        let a = db.insert(Bytes::from_static(b"hello"));
        let b = db.insert(Bytes::from_static(b"world"));
        (db, a, b)
    }

    #[test]
    fn records_returned_entries() {
        let (db, a, b) = fixture();
        let mut recorder = RecordingPreimagesProvider::new(&db);

        let _ = recorder.nearest_lower_preimage_mut(&a).unwrap();
        let _ = recorder.nearest_lower_preimage_mut(&b).unwrap();

        assert_eq!(recorder.recorded().len(), 2);
    }

    #[test]
    fn skips_recording_on_miss() {
        let recorder = RecordingPreimagesProvider::new(EmptyPreimagesProvider);
        let mut recorder = recorder;

        let result = recorder.nearest_lower_preimage_mut(&B256::ZERO).unwrap();
        assert!(result.is_none());
        assert!(recorder.recorded().is_empty());
    }

    #[test]
    fn deduplicates_repeated_lookups() {
        let (db, a, _) = fixture();
        let mut recorder = RecordingPreimagesProvider::new(&db);

        for _ in 0..5 {
            let _ = recorder.nearest_lower_preimage_mut(&a).unwrap();
        }
        assert_eq!(recorder.recorded().len(), 1);
    }

    #[test]
    fn take_recorded_resets_to_empty() {
        let (db, a, _) = fixture();
        let mut recorder = RecordingPreimagesProvider::new(&db);

        let _ = recorder.nearest_lower_preimage_mut(&a).unwrap();
        let taken = recorder.take_recorded();
        assert_eq!(taken.len(), 1);
        assert!(recorder.recorded().is_empty());
    }
}
