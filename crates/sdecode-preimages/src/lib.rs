#![cfg_attr(not(test), warn(unused_crate_dependencies))]

pub use sdecode_preimages_interface::*;

pub mod caches;

pub mod misc;

mod providers;
pub use providers::{
    CachedProvider, EmptyPreimagesProvider, InMemoryPreimages, PreimagesCache, PreimagesCacheInit,
};

mod utils;
