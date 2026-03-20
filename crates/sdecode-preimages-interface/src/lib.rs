#![cfg_attr(not(test), warn(unused_crate_dependencies))]

pub type Image = alloy_primitives::B256;
pub type Preimage = alloy_primitives::Bytes;

mod provider;
pub use provider::{
    BoxedPreimagesProvider, BoxedPreimagesProviderMut, PreimagesProvider, PreimagesProviderMut,
    WrapPreimagesProvider,
};

mod entry;
pub use entry::PreimageEntry;
