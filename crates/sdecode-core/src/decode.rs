use std::error::Error;

use alloy_primitives::{B256, U256};
use quick_impl::quick_impl_all;
use sdecode_preimages::{PreimagesProvider, PreimagesProviderMut, caches::StoragePreimagesCache};

use crate::slot::MAX_STORAGE_OFFSET;

pub trait StorageDecode: Sized {
    type LayoutError: Error;

    fn sdecode_mut<P, E>(
        preimages_provider: &mut P,
        storage_entries: E,
    ) -> Result<Self, StorageError<P::Error, Self::LayoutError>>
    where
        P: PreimagesProviderMut,
        E: IntoIterator<Item = (B256, B256)>;

    fn sdecode<P, E>(
        preimages_provider: P,
        storage_entries: E,
    ) -> Result<Self, StorageError<P::Error, Self::LayoutError>>
    where
        P: PreimagesProvider,
        E: IntoIterator<Item = (B256, B256)>,
    {
        Self::sdecode_mut(
            &mut StoragePreimagesCache::new(preimages_provider, U256::from(MAX_STORAGE_OFFSET)),
            storage_entries,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
#[quick_impl_all(pub const is, pub is_and, pub as_ref, pub as_ref_mut, pub into, pub try_into)]
pub enum StorageError<Provider, Layout> {
    #[error(transparent)]
    Provider(Provider),

    #[error(transparent)]
    Layout(Layout),
}
