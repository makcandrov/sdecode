use std::collections::BTreeMap;

use alloy_primitives::{B256, U256};
use overf::checked;
use quick_impl::quick_impl;

use crate::{
    Image, PreimageEntry, PreimagesProvider, PreimagesProviderMut, WrapPreimagesProvider,
    utils::b256_to_u256,
};

/// A highly efficient cache for a [`PreimagesProvider`], optimized for querying preimages  
/// that are typically located just below the requested image.
///
/// # Optimized Use Case
/// - This cache is designed for cases where queries are **almost always slightly above**   an
///   existing preimage in the provider.
/// - Leveraging the properties of `keccak256`, the cache assumes that once a preimage   exists at
///   `x`, **there are no other preimages within `max_delta`** of `x` with high probability.
/// - The cache maintains a **nearest lower preimage** map, reducing redundant queries   to the
///   underlying provider by storing intervals where no preimages exist.
///
/// # Caching Strategy
/// - The cache stores **confirmed nearest lower preimages** at specific points.
/// - If no preimage is found near a queried image, the cache **remembers empty intervals**   to
///   avoid unnecessary provider lookups in the future.
/// - When a query is far from cached entries, the provider is queried, and both   the result and
///   absence regions are cached efficiently.
///
/// This cache is specifically optimized for storage decoding.
/// For general purpose, this cache is less optimized than
/// [`GeneralPreimagesCache`](super::GeneralPreimagesCache).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[quick_impl]
pub struct StoragePreimagesCache<P> {
    #[quick_impl(pub get = "inner_{}", pub get_mut = "inner_{}_mut", pub into = "into_inner_{}")]
    provider: P,

    /// A cache mapping queried images to their nearest lower preimage.
    ///
    /// If `lower_cache[image]` contains an (optional) entry, then this entry is the nearest lower
    /// preimage that the provider would return for any query in the range `[image, image +
    /// max_delta]`. This leverages the fact that Keccak256 has a high probability of
    /// scattering preimages, ensuring that once a preimage is found, there are no others
    /// nearby within `max_delta`.
    lower_cache: BTreeMap<U256, Option<PreimageEntry>>,

    upper_cache: BTreeMap<U256, Option<PreimageEntry>>,

    #[quick_impl(get_clone = "{}")]
    max_delta: U256,
}

impl<P: PreimagesProvider> StoragePreimagesCache<WrapPreimagesProvider<P>> {
    pub fn new(preimages_provider: P, max_delta: U256) -> Self {
        Self::new_mut(WrapPreimagesProvider(preimages_provider), max_delta)
    }
}

impl<P: PreimagesProviderMut> StoragePreimagesCache<P> {
    pub fn new_mut(preimages_provider: P, max_delta: U256) -> Self {
        let mut lower_cache = BTreeMap::new();
        lower_cache.insert(U256::ZERO, None);
        lower_cache.insert(checked! { U256::MAX - max_delta }, None);

        let mut upper_cache = BTreeMap::new();
        upper_cache.insert(U256::MAX, None);
        upper_cache.insert(max_delta, None);

        Self {
            provider: preimages_provider,
            lower_cache,
            upper_cache,
            max_delta,
        }
    }
}

impl<P: PreimagesProviderMut> PreimagesProviderMut for StoragePreimagesCache<P> {
    type Error = P::Error;

    fn nearest_lower_preimage_mut(
        &mut self,
        image: Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        let image_u256 = b256_to_u256(image);
        let (cache_key, cache_entry) = self
            .lower_cache
            .range(..=image_u256)
            .last()
            .expect("the cache always contains 0, so this cannot be empty");

        // The nearest lower cached entry is found.
        let delta_to_cache = checked! { image_u256 - *cache_key};

        if delta_to_cache <= self.max_delta() {
            // The cached entry is within `max_delta`, so we return it immediately.
            return Ok(cache_entry.clone());
        }

        // The cached entry is too far, so we query the provider.
        let provider_entry = self.provider.nearest_lower_preimage_mut(image)?;

        let provider_key = provider_entry
            .as_ref()
            .map_or(U256::ZERO, PreimageEntry::image_u256);

        if let Some(provider_entry) = &provider_entry {
            if cache_entry
                .as_ref()
                .is_none_or(|entry| entry.image_u256() != provider_key)
            {
                // If this entry isn't already cached, we insert it.
                self.lower_cache
                    .insert(provider_key, Some(provider_entry.clone()));
            }
        }

        let delta_to_provider = checked! { image_u256 - provider_key};
        if delta_to_provider <= self.max_delta() {
            // The provider entry is close enough, and we have already cached it.
            Ok(provider_entry)
        } else {
            // The provider entry is farther than `max_delta`, meaning there are no preimages
            // in the range `[image_u256 - max_delta, image_u256]`. We cache this information
            // to prevent redundant provider queries for future lookups within this range.
            self.lower_cache.insert(
                checked! { image_u256 - self.max_delta() },
                provider_entry.clone(),
            );

            // Now, we attempt to optimize future queries by finding a relevant upper-side entry.
            let next_image_u256 = image_u256.saturating_add(self.max_delta());
            let next_provider_entry = self
                .provider
                .nearest_lower_preimage_mut(B256::from(next_image_u256))?;

            if let Some(next_provider_entry) = next_provider_entry {
                let next_entry_image_u256 = next_provider_entry.image_u256();
                if next_entry_image_u256 <= image_u256 {
                    // The found entry is valid for all queries in `[image_u256, image_u256 +
                    // max_delta]`, so we cache it under `image_u256`.
                    self.lower_cache
                        .insert(image_u256, Some(next_provider_entry.clone()));

                    Ok(Some(next_provider_entry))
                } else {
                    // There is an entry between `image_u256` and `image_u256 + max_delta`.

                    // We cache that `[next_entry_image_u256 - max_delta, next_entry_image_u256]`
                    // resolves to the original entry.
                    self.lower_cache.insert(
                        checked! { next_entry_image_u256 - self.max_delta() },
                        provider_entry.clone(),
                    );

                    // We cache that `[next_entry_image_u256, next_entry_image_u256 + max_delta]`
                    // resolves to the next entry.
                    self.lower_cache
                        .insert(next_entry_image_u256, Some(next_provider_entry));

                    // Returning the nearest provider entry found for `image_u256`.
                    Ok(provider_entry)
                }
            } else {
                assert!(provider_entry.is_none());
                self.lower_cache.insert(image_u256, None);
                Ok(None)
            }
        }
    }

    fn nearest_upper_preimage_mut(
        &mut self,
        image: Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        self.provider.nearest_upper_preimage_mut(image)
    }
}

#[cfg(test)]
mod tests {
    use crate::{MemoryPreimagesProvider, misc::CounterPreimagesProviderMut};

    use super::*;

    #[test]
    fn test_storage_preimages_cache() {
        let max_delta = U256::from(0xffffffffffffusize);
        let db = MemoryPreimagesProvider::random_filled(10);
        let db_counter = CounterPreimagesProviderMut::new(&db);
        let mut cache = StoragePreimagesCache::new_mut(db_counter, max_delta);

        const N: usize = 50;
        for _ in 0..N {
            let random_key = B256::random();
            let db_response = db.nearest_lower_preimage(random_key).unwrap();
            let cache_response = cache.nearest_lower_preimage_mut(random_key).unwrap();

            assert_eq!(db_response, cache_response);
        }

        let accessses = cache.inner_provider().accesses();

        // dbg!(cache);

        println!("{N} cache queries\n{accessses} db accesses");
    }
}
