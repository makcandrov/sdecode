use std::collections::BTreeMap;

use alloy_preimages::{
    Image, PreimageEntry, PreimagesProviderMut,
    providers::{CachedProvider, PreimagesCache, PreimagesCacheInit},
};
use alloy_primitives::U256;

use crate::utils::{B256_MAX, b256_to_u256};

/// A [`CachedProvider`] using a [`GeneralCache`] for general-purpose preimage caching.
pub type GeneralCachedProvider<P> = CachedProvider<P, GeneralCache>;

/// A general-purpose preimages cache that makes no assumptions about query patterns.
///
/// This cache tracks explored regions of the image space and caches discovered preimage
/// locations. Each cache miss queries the provider for both `nearest_lower` and
/// `nearest_upper`, revealing the interval between two adjacent preimages. This interval
/// is marked as explored, and future queries within explored regions are answered directly
/// from the cache.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct GeneralCache {
    /// Known preimage locations discovered through provider queries.
    preimages: BTreeMap<U256, PreimageEntry>,
    /// Explored ranges: maps range_start -> range_end (both inclusive).
    /// Within an explored range, all preimage locations are known and stored in `preimages`.
    explored: BTreeMap<U256, U256>,
}

impl GeneralCache {
    pub fn new<P: PreimagesProviderMut>(provider: &mut P) -> Result<Self, P::Error> {
        let mut cache = Self {
            preimages: BTreeMap::new(),
            explored: BTreeMap::new(),
        };

        // Query boundary preimages to establish initial explored ranges.
        let min_entry = provider.nearest_upper_preimage_mut(&Image::ZERO)?;
        let max_entry = provider.nearest_lower_preimage_mut(&B256_MAX)?;

        // [0, min_image] is explored: no preimages in [0, min_image).
        let min_image = min_entry.as_ref().map_or(U256::MAX, |e| e.image_u256());
        cache.add_explored(U256::ZERO, min_image);

        // [max_image, MAX] is explored: no preimages in (max_image, MAX].
        let max_image = max_entry.as_ref().map_or(U256::ZERO, |e| e.image_u256());
        cache.add_explored(max_image, U256::MAX);

        if let Some(entry) = min_entry {
            cache.preimages.insert(entry.image_u256(), entry);
        }
        if let Some(entry) = max_entry {
            cache.preimages.insert(entry.image_u256(), entry);
        }

        Ok(cache)
    }

    /// Returns the explored range containing the point, if any.
    fn explored_range(&self, point: U256) -> Option<(U256, U256)> {
        self.explored
            .range(..=point)
            .next_back()
            .filter(|(_, end)| point <= **end)
            .map(|(&start, end)| (start, *end))
    }

    /// Merges a new explored range [start, end] into the explored map,
    /// coalescing with any overlapping or adjacent existing ranges.
    fn add_explored(&mut self, mut start: U256, mut end: U256) {
        // Check for a range ending at or adjacent to `start`.
        if let Some((&s, &e)) = self.explored.range(..=start).next_back()
            && (e >= start || (start > U256::ZERO && e >= start - U256::from(1)))
        {
            start = s;
            end = end.max(e);
            self.explored.remove(&s);
        }

        // Absorb all ranges starting within [start, end] or adjacent (at end + 1).
        let upper = if end < U256::MAX {
            end + U256::from(1)
        } else {
            U256::MAX
        };
        let overlapping: Vec<(U256, U256)> = self
            .explored
            .range(start..=upper)
            .map(|(&s, &e)| (s, e))
            .collect();
        for (s, e) in overlapping {
            end = end.max(e);
            self.explored.remove(&s);
        }

        self.explored.insert(start, end);
    }

    /// Caches the results of a provider query and marks the discovered interval as explored.
    fn cache_results(&mut self, lower: &Option<PreimageEntry>, upper: &Option<PreimageEntry>) {
        let range_start = lower.as_ref().map_or(U256::ZERO, |e| e.image_u256());
        let range_end = upper.as_ref().map_or(U256::MAX, |e| e.image_u256());

        if let Some(entry) = lower {
            self.preimages
                .entry(entry.image_u256())
                .or_insert_with(|| entry.clone());
        }
        if let Some(entry) = upper {
            self.preimages
                .entry(entry.image_u256())
                .or_insert_with(|| entry.clone());
        }

        self.add_explored(range_start, range_end);
    }
}

impl<P: PreimagesProviderMut> PreimagesCacheInit<P> for GeneralCache {
    type Params = ();
    type InitError = P::Error;

    fn new_init(provider: &mut P, (): ()) -> Result<Self, P::Error> {
        Self::new(provider)
    }
}

impl<P: PreimagesProviderMut> PreimagesCache<P> for GeneralCache {
    fn nearest_lower_preimage_mut(
        &mut self,
        provider: &mut P,
        image: &Image,
    ) -> Result<Option<PreimageEntry>, P::Error> {
        let image_u256 = b256_to_u256(*image);

        if let Some((range_start, _)) = self.explored_range(image_u256) {
            return Ok(self
                .preimages
                .range(range_start..=image_u256)
                .next_back()
                .map(|(_, e)| e.clone()));
        }

        let lower = provider.nearest_lower_preimage_mut(image)?;
        let upper = provider.nearest_upper_preimage_mut(image)?;
        self.cache_results(&lower, &upper);

        Ok(lower)
    }

    fn nearest_upper_preimage_mut(
        &mut self,
        provider: &mut P,
        image: &Image,
    ) -> Result<Option<PreimageEntry>, P::Error> {
        let image_u256 = b256_to_u256(*image);

        if let Some((_, range_end)) = self.explored_range(image_u256) {
            return Ok(self
                .preimages
                .range(image_u256..=range_end)
                .next()
                .map(|(_, e)| e.clone()));
        }

        let lower = provider.nearest_lower_preimage_mut(image)?;
        let upper = provider.nearest_upper_preimage_mut(image)?;
        self.cache_results(&lower, &upper);

        Ok(upper)
    }
}

#[cfg(test)]
mod tests {
    use alloy_preimages::{
        Preimage, PreimagesProvider,
        providers::{InMemoryPreimages, PreimagesCounter},
    };
    use alloy_primitives::B256;

    use super::*;

    #[test]
    fn test_general_cache() {
        let mut db = InMemoryPreimages::new();

        for _ in 0..10 {
            db.insert(Preimage::copy_from_slice(&Image::random().0));
        }

        let mut db_counter = PreimagesCounter::new(&db);
        let mut cache = GeneralCache::new_init(&mut db_counter, ()).unwrap();

        const N: usize = 50;
        for _ in 0..N {
            let random_key = B256::random();

            let db_lower = db.nearest_lower_preimage(&random_key).unwrap();
            let cache_lower = cache
                .nearest_lower_preimage_mut(&mut db_counter, &random_key)
                .unwrap();
            assert_eq!(db_lower, cache_lower);

            let db_upper = db.nearest_upper_preimage(&random_key).unwrap();
            let cache_upper = cache
                .nearest_upper_preimage_mut(&mut db_counter, &random_key)
                .unwrap();
            assert_eq!(db_upper, cache_upper);
        }

        let accesses = db_counter.accesses();
        println!("{N} cache queries (lower + upper)\n{accesses} db accesses");
    }

    #[test]
    fn test_general_cache_empty_provider() {
        let db = InMemoryPreimages::new();
        let mut db_counter = PreimagesCounter::new(&db);
        let mut cache = GeneralCache::new_init(&mut db_counter, ()).unwrap();

        for _ in 0..10 {
            let random_key = B256::random();
            assert_eq!(
                cache
                    .nearest_lower_preimage_mut(&mut db_counter, &random_key)
                    .unwrap(),
                None,
            );
            assert_eq!(
                cache
                    .nearest_upper_preimage_mut(&mut db_counter, &random_key)
                    .unwrap(),
                None,
            );
        }

        // Init uses 2 queries. After that, entire space is explored (empty provider).
        // All subsequent queries are cache hits.
        assert_eq!(db_counter.accesses(), 2);
    }
}
