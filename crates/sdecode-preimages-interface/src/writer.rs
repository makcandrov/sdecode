use std::{error::Error, iter::once};

use quick_impl::quick_impl;

use crate::PreimageEntry;

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
        preimages: impl IntoIterator<Item = &'a PreimageEntry>,
    ) -> Result<(), Self::Error>;

    /// Persists a single preimage entry.
    ///
    /// The default implementation delegates to [`write_preimages`](Self::write_preimages).
    fn write_preimage_entry(&self, preimage: &PreimageEntry) -> Result<(), Self::Error> {
        self.write_preimages(once(preimage))
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
        preimages: impl IntoIterator<Item = &'a PreimageEntry>,
    ) -> Result<(), Self::Error>;

    /// Persists a single preimage entry.
    ///
    /// The default implementation delegates to [`write_preimages_mut`](Self::write_preimages_mut).
    fn write_preimage_entry_mut(&mut self, preimage: &PreimageEntry) -> Result<(), Self::Error> {
        self.write_preimages_mut(once(preimage))
    }
}

/// Wraps a [`PreimagesWriter`] to implement [`PreimagesWriterMut`].
///
/// This is useful when you have a shared writer but need to pass it to an API that requires
/// [`PreimagesWriterMut`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[quick_impl(impl From)]
pub struct WrapPreimagesWriter<W>(pub W);

impl<W> WrapPreimagesWriter<W> {
    /// Creates a new wrapper around the given writer.
    pub const fn new(writer: W) -> Self {
        Self(writer)
    }
}

impl<W: PreimagesWriter> PreimagesWriterMut for WrapPreimagesWriter<W> {
    type Error = W::Error;

    #[inline(always)]
    fn write_preimages_mut<'a>(
        &mut self,
        preimages: impl IntoIterator<Item = &'a PreimageEntry>,
    ) -> Result<(), Self::Error> {
        self.0.write_preimages(preimages)
    }
}

