use std::error::Error;

use alloy_primitives::B256;
use quick_impl::quick_impl_all;
use sdecode_preimages::{
    CachedProvider, PreimagesProvider, PreimagesProviderMut,
    caches::StorageCache,
};

use crate::MAX_STORAGE_OFFSET_U256;

pub trait StorageDecode: Sized {
    type LayoutError: Error;

    fn sdecode_mut<P, E>(
        preimages_provider: &mut P,
        storage_entries: E,
    ) -> SdecodeMutResult<Self, P>
    where
        P: PreimagesProviderMut,
        E: IntoIterator<Item = (B256, B256)>;

    fn sdecode<P, E>(preimages_provider: P, storage_entries: E) -> SdecodeResult<Self, P>
    where
        P: PreimagesProvider,
        E: IntoIterator<Item = (B256, B256)>,
    {
        let cache = StorageCache::new(MAX_STORAGE_OFFSET_U256);
        let mut cached_provider = CachedProvider::from_raw(preimages_provider, cache);
        Self::sdecode_mut(&mut cached_provider, storage_entries)
    }
}

pub type SdecodeResult<T, Provider> = Result<
    T,
    StorageError<<Provider as PreimagesProvider>::Error, <T as StorageDecode>::LayoutError>,
>;

pub type SdecodeMutResult<T, Provider> = Result<
    T,
    StorageError<<Provider as PreimagesProviderMut>::Error, <T as StorageDecode>::LayoutError>,
>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
#[quick_impl_all(pub const is, pub is_and, pub as_ref, pub as_ref_mut, pub into, pub try_into)]
pub enum StorageError<Provider, Layout> {
    #[error(transparent)]
    Provider(Provider),

    #[error(transparent)]
    Layout(Layout),
}
