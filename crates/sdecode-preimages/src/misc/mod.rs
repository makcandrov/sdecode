//! Miscellaneous provider wrappers.
//!
//! These adapters compose around any [`PreimagesProvider`](crate::PreimagesProvider) or
//! [`PreimagesProviderMut`](crate::PreimagesProviderMut) to add cross-cutting behavior such as
//! tracking access counts or capturing visited entries into an in-memory store.

mod counting;
pub use counting::CountingPreimagesProvider;

mod recording;
pub use recording::RecordingPreimagesProvider;
