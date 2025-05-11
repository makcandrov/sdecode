#![cfg_attr(not(test), warn(unused_crate_dependencies))]

pub mod caches;

mod interfaces;
pub use interfaces::{
    BoxedPreimagesProvider, BoxedPreimagesProviderMut, PreimagesProvider, PreimagesProviderMut,
    WrapPreimagesProvider,
};

pub mod misc;

mod providers;
pub use providers::{EmptyPreimagesProvider, MemoryPreimagesProvider};

mod types;
pub use types::{Image, Preimage, PreimageEntry};

mod utils;
