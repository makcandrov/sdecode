use std::{collections::BTreeMap, convert::Infallible};

use alloy_primitives::{B256, U256};
use overf::checked;

use crate::{
    CachedProvider, Image, PreimageEntry, PreimagesCache, PreimagesCacheInit, PreimagesProviderMut,
    utils::b256_to_u256,
};

pub const STORAGE_CACHE_DEFAULT_MAX_DELTA: U256 = U256::from_be_slice(&u32::MAX.to_be_bytes());

/// A [`CachedProvider`] using a [`StorageCache`] for storage-optimized preimage caching.
pub type StorageCachedProvider<P> = CachedProvider<P, StorageCache>;

/// A highly efficient cache for a [`PreimagesCache`], optimized for querying preimages
/// that are typically located just below the requested image.
///
/// # Optimized use case
///
/// This cache is designed for **storage decoding**, where queries are almost always
/// slightly above an existing preimage (e.g., a keccak256 hash + small storage slot offset).
/// It leverages the keccak256 scattering property: once a preimage exists at `x`, there are
/// **no other preimages within `max_delta`** of `x` with high probability.
///
/// # Caching strategy
///
/// The cache maintains a [`BTreeMap`] of confirmed nearest-lower-preimage intervals.
/// Each entry `(k, v)` means: for any query in `[k, k + max_delta]`, the nearest lower
/// preimage is `v`. This is valid because the scattering assumption guarantees at most one
/// preimage per `max_delta`-sized window.
///
/// On a cache miss, the provider is queried and the result is cached. When the nearest
/// preimage is far from the query (farther than `max_delta`), the cache also performs a
/// **lookahead** query at `image + max_delta` to proactively cache the surrounding region,
/// reducing future provider accesses.
///
/// # Scattering assumption
///
/// **This cache assumes that no two preimages in the provider have images within
/// `max_delta` of each other.** For keccak256 outputs, which are uniformly distributed over
/// 2^256 values, the probability of two images being within `max_delta` is approximately
/// `k² * max_delta / 2^256` for `k` preimages. With typical parameters (`max_delta` ~2^48,
/// `k` ~10^6), this probability is negligible (~2^-168).
///
/// Violations of this assumption may cause **incorrect cache hits** where the cached entry
/// is valid (on the correct side of the query) but is not the nearest preimage.
///
/// # Boundary sentinels
///
/// The cache is initialized with sentinel entries at `0` and `MAX - max_delta` that return
/// `None`. This assumes no preimages exist near `0` or near `U256::MAX`, which holds with
/// overwhelming probability for keccak256 outputs.
///
/// # `nearest_upper_preimage_mut`
///
/// Upper preimage queries are **not cached** and delegate directly to the provider. This is
/// acceptable because the primary use case (storage decoding) overwhelmingly queries
/// nearest-lower.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StorageCache {
    /// Interval cache for nearest-lower-preimage queries.
    ///
    /// Each entry `(k, v)` means: for any query in `[k, k + max_delta]`, the nearest lower
    /// preimage is `v`. This leverages the keccak256 scattering property — once a preimage
    /// is found, there are no others nearby within `max_delta`.
    lower_cache: BTreeMap<U256, Option<PreimageEntry>>,

    /// The maximum distance between a query image and its expected nearest preimage.
    ///
    /// Determines the size of cached intervals: each cache entry covers a window of
    /// `max_delta` values. Must be chosen so that no two preimages in the provider have
    /// images within `max_delta` of each other (the scattering assumption).
    max_delta: U256,
}

impl StorageCache {
    pub fn new(max_delta: U256) -> Self {
        let mut lower_cache = BTreeMap::new();

        // Sentinel at 0: for queries in [0, max_delta], nearest_lower = None.
        // Relies on the scattering assumption: no keccak256 preimage has an image
        // in [0, max_delta] (probability ~max_delta / 2^256, negligible).
        lower_cache.insert(U256::ZERO, None);

        // Sentinel near MAX: for queries in [MAX - max_delta, MAX], nearest_lower = None.
        // Prevents overflow when computing `image + max_delta` in the lookahead step,
        // and relies on the same scattering assumption for correctness.
        lower_cache.insert(checked! { U256::MAX - max_delta }, None);

        Self {
            lower_cache,
            max_delta,
        }
    }

    pub fn max_delta(&self) -> U256 {
        self.max_delta
    }
}

impl<P: PreimagesProviderMut> PreimagesCacheInit<P> for StorageCache {
    type Params = U256;
    type InitError = Infallible;

    fn new_init(_provider: &mut P, max_delta: U256) -> Result<Self, Infallible> {
        Ok(Self::new(max_delta))
    }
}

impl<P: PreimagesProviderMut> PreimagesCache<P> for StorageCache {
    fn nearest_lower_preimage_mut(
        &mut self,
        provider: &mut P,
        image: &Image,
    ) -> Result<Option<PreimageEntry>, P::Error> {
        let image_u256 = b256_to_u256(*image);
        let (cache_key, cache_entry) = self
            .lower_cache
            .range(..=image_u256)
            .last()
            .expect("the cache always contains 0, so this cannot be empty");

        let delta_to_cache = checked! { image_u256 - *cache_key};

        if delta_to_cache <= self.max_delta() {
            // The query falls within a cached interval — return the cached answer.
            return Ok(cache_entry.clone());
        }

        // Cache miss: query the provider for the nearest lower preimage.
        let provider_entry = provider.nearest_lower_preimage_mut(image)?;

        let provider_key = provider_entry
            .as_ref()
            .map_or(U256::ZERO, PreimageEntry::image_u256);

        if let Some(provider_entry) = &provider_entry
            && cache_entry
                .as_ref()
                .is_none_or(|entry| entry.image_u256() != provider_key)
        {
            // Cache the preimage at its own location: queries in
            // [provider_key, provider_key + max_delta] now resolve to this entry.
            self.lower_cache
                .insert(provider_key, Some(provider_entry.clone()));
        }

        let delta_to_provider = checked! { image_u256 - provider_key};
        if delta_to_provider <= self.max_delta() {
            // The provider entry is within max_delta — already cached above.
            Ok(provider_entry)
        } else {
            // The nearest preimage is farther than max_delta. This means the interval
            // [image - max_delta, image] contains no preimages (by scattering, consecutive
            // preimages are > max_delta apart). Cache this empty interval.
            self.lower_cache.insert(
                checked! { image_u256 - self.max_delta() },
                provider_entry.clone(),
            );

            // Lookahead: query at image + max_delta to proactively discover and cache
            // the next preimage above the current query point.
            let next_image_u256 = image_u256.saturating_add(self.max_delta());
            let next_provider_entry =
                provider.nearest_lower_preimage_mut(&B256::from(next_image_u256))?;

            if let Some(next_provider_entry) = next_provider_entry {
                let next_entry_image_u256 = next_provider_entry.image_u256();
                if next_entry_image_u256 <= image_u256 {
                    // The lookahead found no new preimage above image — the result is
                    // still at or below image (same as provider_entry, under scattering).
                    // Cache: queries in [image, image + max_delta] resolve to this entry.
                    self.lower_cache
                        .insert(image_u256, Some(next_provider_entry.clone()));

                    Ok(Some(next_provider_entry))
                } else {
                    // The lookahead found a preimage between image and image + max_delta.
                    // Under scattering, this is the next preimage after provider_entry.

                    // Cache the gap: [next_entry - max_delta, next_entry] resolves to
                    // provider_entry (the preimage before the gap).
                    self.lower_cache.insert(
                        checked! { next_entry_image_u256 - self.max_delta() },
                        provider_entry.clone(),
                    );

                    // Cache the next preimage's interval.
                    self.lower_cache
                        .insert(next_entry_image_u256, Some(next_provider_entry));

                    Ok(provider_entry)
                }
            } else {
                // The lookahead returned None — no preimages exist at all up to
                // image + max_delta. The original query must also be None.
                assert!(provider_entry.is_none());
                self.lower_cache.insert(image_u256, None);
                Ok(None)
            }
        }
    }

    /// Not cached — delegates directly to the provider.
    ///
    /// This cache is optimized for nearest-lower queries (the storage decoding hot path).
    /// Upper queries bypass the cache entirely.
    fn nearest_upper_preimage_mut(
        &mut self,
        provider: &mut P,
        image: &Image,
    ) -> Result<Option<PreimageEntry>, P::Error> {
        provider.nearest_upper_preimage_mut(image)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        InMemoryPreimages, Preimage, PreimagesProvider, PreimagesProviderMut,
        misc::CounterPreimagesProviderMut,
    };

    use super::*;

    #[test]
    fn test_storage_preimages_cache() {
        let mut db = InMemoryPreimages::new();

        for _ in 0..10 {
            db.insert(Preimage::copy_from_slice(&Image::random().0));
        }

        let db_counter = CounterPreimagesProviderMut::new(&db);
        let mut cache =
            StorageCachedProvider::new_mut(db_counter, STORAGE_CACHE_DEFAULT_MAX_DELTA).unwrap();

        const N: usize = 50;
        for _ in 0..N {
            let random_key = B256::random();
            let db_response = db.nearest_lower_preimage(&random_key).unwrap();
            let cache_response = cache.nearest_lower_preimage_mut(&random_key).unwrap();

            assert_eq!(db_response, cache_response);
        }

        let accesses = cache.provider().accesses();

        println!("{N} cache queries\n{accesses} db accesses");
    }

    #[test]
    fn test_storage_cache_empty_provider() {
        let db = InMemoryPreimages::new();
        let db_counter = CounterPreimagesProviderMut::new(&db);
        let mut cache =
            StorageCachedProvider::new_mut(db_counter, STORAGE_CACHE_DEFAULT_MAX_DELTA).unwrap();

        for _ in 0..10 {
            let random_key = B256::random();
            assert_eq!(cache.nearest_lower_preimage_mut(&random_key).unwrap(), None);
        }
    }

    #[test]
    fn test_storage_cache_upper_delegates() {
        let mut db = InMemoryPreimages::new();

        for _ in 0..10 {
            db.insert(Preimage::copy_from_slice(&Image::random().0));
        }

        let db_counter = CounterPreimagesProviderMut::new(&db);
        let mut cache =
            StorageCachedProvider::new_mut(db_counter, STORAGE_CACHE_DEFAULT_MAX_DELTA).unwrap();

        for _ in 0..10 {
            let random_key = B256::random();
            let db_response = db.nearest_upper_preimage(&random_key).unwrap();
            let cache_response = cache.nearest_upper_preimage_mut(&random_key).unwrap();
            assert_eq!(db_response, cache_response);
        }

        // Every upper query hits the provider (uncached).
        assert_eq!(cache.provider().accesses(), 10);
    }

    #[test]
    fn test_storage_cache_nearby_queries_hit() {
        let mut db = InMemoryPreimages::new();

        for _ in 0..10 {
            db.insert(Preimage::copy_from_slice(&Image::random().0));
        }

        let db_counter = CounterPreimagesProviderMut::new(&db);
        let mut cache =
            StorageCachedProvider::new_mut(db_counter, STORAGE_CACHE_DEFAULT_MAX_DELTA).unwrap();

        // Query at a random point to populate the cache.
        let base_key = B256::random();
        let _ = cache.nearest_lower_preimage_mut(&base_key).unwrap();
        let accesses_after_first = cache.provider().accesses();

        // Query at the same point again — should be a cache hit.
        let response1 = cache.nearest_lower_preimage_mut(&base_key).unwrap();
        assert_eq!(cache.provider().accesses(), accesses_after_first);

        // Verify correctness.
        let db_response = db.nearest_lower_preimage(&base_key).unwrap();
        assert_eq!(response1, db_response);
    }
}
