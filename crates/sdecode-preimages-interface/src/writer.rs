use std::{error::Error, iter::once};

use crate::PreimageEntryRef;

/// Write-only sink for persisting keccak256 preimage entries.
///
/// Takes `&self`, so implementations are expected to handle synchronization internally
/// (e.g. via interior mutability).
#[auto_impl::auto_impl(&, &mut, Box, Arc, Rc)]
pub trait PreimagesWriter {
    /// The error type returned by write operations.
    type Error: Error;

    /// Persists all preimage entries yielded by the iterator.
    fn write_preimages<'a>(
        &self,
        preimages: impl IntoIterator<Item = impl Into<PreimageEntryRef<'a>>>,
    ) -> Result<(), Self::Error>;

    /// Persists a single preimage entry.
    ///
    /// The default implementation delegates to [`write_preimages`](Self::write_preimages).
    #[inline]
    fn write_preimage_entry<'a>(
        &self,
        entry: impl Into<PreimageEntryRef<'a>>,
    ) -> Result<(), Self::Error> {
        self.write_preimages(once(entry))
    }
}

/// Like [`PreimagesWriter`], but takes `&mut self`, allowing implementations to update internal
/// state on each write.
#[auto_impl::auto_impl(&mut, Box)]
pub trait PreimagesWriterMut {
    /// The error type returned by write operations.
    type Error: Error;

    /// Persists all preimage entries yielded by the iterator.
    fn write_preimages_mut<'a>(
        &mut self,
        preimages: impl IntoIterator<Item = impl Into<PreimageEntryRef<'a>>>,
    ) -> Result<(), Self::Error>;

    /// Persists a single preimage entry.
    ///
    /// The default implementation delegates to [`write_preimages_mut`](Self::write_preimages_mut).
    #[inline]
    fn write_preimage_entry_mut<'a>(
        &mut self,
        entry: impl Into<PreimageEntryRef<'a>>,
    ) -> Result<(), Self::Error> {
        self.write_preimages_mut(once(entry))
    }
}

/// Wraps a [`PreimagesWriter`] to implement [`PreimagesWriterMut`].
///
/// This is useful when you have a shared writer but need to pass it to an API that requires
/// [`PreimagesWriterMut`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WrapPreimagesWriter<W>(pub W);

impl<W> From<W> for WrapPreimagesWriter<W> {
    fn from(writer: W) -> Self {
        Self(writer)
    }
}

impl<W> WrapPreimagesWriter<W> {
    /// Creates a new wrapper around the given writer.
    pub const fn new(writer: W) -> Self {
        Self(writer)
    }
}

impl<W: PreimagesWriter> PreimagesWriterMut for WrapPreimagesWriter<W> {
    type Error = W::Error;

    #[inline]
    fn write_preimages_mut<'a>(
        &mut self,
        preimages: impl IntoIterator<Item = impl Into<PreimageEntryRef<'a>>>,
    ) -> Result<(), Self::Error> {
        self.0.write_preimages(preimages)
    }

    #[inline]
    fn write_preimage_entry_mut<'a>(
        &mut self,
        entry: impl Into<PreimageEntryRef<'a>>,
    ) -> Result<(), Self::Error> {
        self.0.write_preimage_entry(entry)
    }
}
