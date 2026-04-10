#![cfg_attr(not(test), warn(unused_crate_dependencies))]

//! Traits and types for working with keccak256 preimages.
//!
//! This crate defines the [`PreimageEntry`] type that pairs a keccak256 hash ([`Image`]) with its
//! corresponding [`Preimage`] bytes, along with provider traits ([`PreimagesProvider`] and
//! [`PreimagesProviderMut`]) for looking up preimages by their hash.

/// A keccak256 hash digest.
pub type Image = alloy_primitives::B256;

/// The raw bytes whose keccak256 hash produces an [`Image`].
pub type Preimage = alloy_primitives::Bytes;


mod entry;
pub use entry::PreimageEntry;

mod provider;
pub use provider::{
    BoxedPreimagesProvider, BoxedPreimagesProviderMut, PreimagesProvider, PreimagesProviderMut,
    WrapPreimagesProvider,
};

mod writer;
pub use writer::{PreimagesWriter, PreimagesWriterMut, WrapPreimagesWriter};
