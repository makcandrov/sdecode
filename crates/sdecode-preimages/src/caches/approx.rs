use alloy_primitives::{B256, FixedBytes};
use hashbrown::HashMap;

use crate::{
    CachedProvider, PreimageEntry, PreimagesCache, PreimagesCacheInit, PreimagesProviderMut,
    utils::B256_MAX,
};

/// Default number of image prefix bytes used as the cache key.
///
/// 16 bytes (128 bits) keeps collision probability below 2⁻³² for up to 2⁴⁸ preimages,
/// which is safe for any practical dataset.
pub const APPROX_CACHE_DEFAULT_PREFIX_LEN: usize = 16;

/// A [`CachedProvider`] using an [`ApproxCache`] for approximate O(1) preimage lookups.
pub type ApproxCachedProvider<P, const N: usize = APPROX_CACHE_DEFAULT_PREFIX_LEN> =
    CachedProvider<P, ApproxCache<N>>;

/// A fast preimages cache that uses the first `N` bytes of an image as a hash-map key,
/// providing O(1) lookups instead of O(log n) sorted lookups.
///
/// # How it works
///
/// Each preimage entry discovered through a provider query is stored in a [`HashMap`]
/// keyed by the first `N` bytes of its **image**. On subsequent queries, the first `N`
/// bytes of the query image are used for lookup. If a cached entry is found, it is
/// returned directly without hitting the provider.
///
/// # Correctness assumption: no prefix collisions
///
/// This cache is **only correct** when no two preimages in the provider share the same
/// `N`-byte image prefix. Under this assumption, each prefix bucket contains at most one
/// preimage, so any cache hit returns the unique preimage in that bucket — which is
/// guaranteed to be the nearest in the queried direction.
///
/// Since keccak256 outputs are uniformly distributed, the collision probability follows the
/// birthday bound: for `k` preimages and `N` prefix bytes, the probability of at least one
/// collision is approximately `k² / 2^(8*N + 1)`. Some reference values:
///
/// | `N` | Prefix bits | Safe up to `k` preimages (collision prob < 2⁻³²) |
/// |-----|-------------|---------------------------------------------------|
/// |  4  |    32       | ~1 (NOT safe for any real use)                     |
/// |  8  |    64       | ~65 536                                            |
/// | 12  |    96       | ~4 billion                                         |
/// | 16  |   128       | ~2⁴⁸ (safe for any practical dataset)              |
///
/// # Collision detection
///
/// Runtime assertions check that a cached entry is on the correct side of the query
/// (i.e. `<=` for nearest-lower, `>=` for nearest-upper). This catches "wrong-side"
/// collisions where two preimages share a prefix and the cached one is in the wrong
/// direction. However, "same-side" collisions — where the cached entry is valid but a
/// closer preimage exists in the same bucket — are **not detected** and would silently
/// return an incorrect (non-nearest) result.
///
/// Choose `N` large enough for the expected dataset to make collisions negligible.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct ApproxCache<const N: usize = APPROX_CACHE_DEFAULT_PREFIX_LEN> {
    /// Maps the first `N` bytes of a preimage's image to the full entry.
    cache: HashMap<FixedBytes<N>, PreimageEntry>,
    /// The smallest known image in the provider, used to short-circuit queries below it.
    min: B256,
    /// The largest known image in the provider, used to short-circuit queries above it.
    max: B256,
}

impl<const N: usize> ApproxCache<N> {
    pub fn new<P: PreimagesProviderMut>(provider: &mut P) -> Result<Self, P::Error> {
        let min = provider
            .nearest_upper_preimage_mut(B256::ZERO)?
            .map(|entry| entry.image())
            .unwrap_or(B256_MAX);
        let max = provider
            .nearest_lower_preimage_mut(B256_MAX)?
            .map(|entry| entry.image())
            .unwrap_or(B256::ZERO);
        Ok(Self {
            cache: HashMap::new(),
            min,
            max,
        })
    }
}

impl<const N: usize, P: PreimagesProviderMut> PreimagesCacheInit<P> for ApproxCache<N> {
    type Params = ();
    type InitError = P::Error;

    fn new_init(provider: &mut P, (): ()) -> Result<Self, P::Error> {
        Self::new(provider)
    }
}

impl<const N: usize, P: PreimagesProviderMut> PreimagesCache<P> for ApproxCache<N> {
    fn nearest_lower_preimage_mut(
        &mut self,
        provider: &mut P,
        image: crate::Image,
    ) -> Result<Option<PreimageEntry>, <P as PreimagesProviderMut>::Error> {
        if image < self.min {
            return Ok(None);
        }
        if let Some(entry) = self.cache.get(&image[..N]) {
            // Under the no-collision assumption, the unique preimage in this prefix bucket
            // must be at or below the query image to be a valid nearest-lower result.
            // If it's above, two preimages share a prefix — N is too small.
            assert!(
                entry.image() <= image,
                "prefix collision detected: cached entry is above the query, chosen N ({N}) is too small",
            );
            Ok(Some(entry.clone()))
        } else {
            let entry = provider
                .nearest_lower_preimage_mut(image)?
                .expect("image is greater than or equal to `min`, so a preimage must exist");
            let key = FixedBytes::<N>::from_slice(&entry.image()[..N]);
            self.cache.insert(key, entry.clone());
            Ok(Some(entry))
        }
    }

    fn nearest_upper_preimage_mut(
        &mut self,
        provider: &mut P,
        image: crate::Image,
    ) -> Result<Option<PreimageEntry>, <P as PreimagesProviderMut>::Error> {
        if image > self.max {
            return Ok(None);
        }
        if let Some(entry) = self.cache.get(&image[..N]) {
            // Under the no-collision assumption, the unique preimage in this prefix bucket
            // must be at or above the query image to be a valid nearest-upper result.
            // If it's below, two preimages share a prefix — N is too small.
            assert!(
                entry.image() >= image,
                "prefix collision detected: cached entry is below the query, chosen N ({N}) is too small",
            );
            Ok(Some(entry.clone()))
        } else {
            let entry = provider
                .nearest_upper_preimage_mut(image)?
                .expect("image is less than or equal to `max`, so a preimage must exist");
            let key = FixedBytes::<N>::from_slice(&entry.image()[..N]);
            self.cache.insert(key, entry.clone());
            Ok(Some(entry))
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::B256;

    use crate::{
        Image, MemoryPreimagesProvider, Preimage, PreimagesProvider,
        misc::CounterPreimagesProviderMut,
    };

    use super::*;

    #[test]
    fn test_approx_cache() {
        let mut db = MemoryPreimagesProvider::new();

        for _ in 0..10 {
            db.insert(Preimage::copy_from_slice(&Image::random().0));
        }

        // N=16 is very safe for 10 preimages (128 prefix bits, collision prob ~2⁻¹¹³).
        let mut db_counter = CounterPreimagesProviderMut::new(&db);
        let mut cache = ApproxCache::<16>::new_init(&mut db_counter, ()).unwrap();

        const QUERIES: usize = 50;
        for _ in 0..QUERIES {
            let random_key = B256::random();

            let db_lower = db.nearest_lower_preimage(random_key).unwrap();
            let cache_lower = cache
                .nearest_lower_preimage_mut(&mut db_counter, random_key)
                .unwrap();
            assert_eq!(db_lower, cache_lower);

            let db_upper = db.nearest_upper_preimage(random_key).unwrap();
            let cache_upper = cache
                .nearest_upper_preimage_mut(&mut db_counter, random_key)
                .unwrap();
            assert_eq!(db_upper, cache_upper);
        }

        let accesses = db_counter.accesses();
        println!("{QUERIES} cache queries (lower + upper)\n{accesses} db accesses");
    }

    #[test]
    fn test_approx_cache_empty_provider() {
        let db = MemoryPreimagesProvider::new();
        let mut db_counter = CounterPreimagesProviderMut::new(&db);
        let mut cache = ApproxCache::<16>::new_init(&mut db_counter, ()).unwrap();

        for _ in 0..10 {
            let random_key = B256::random();
            assert_eq!(
                cache
                    .nearest_lower_preimage_mut(&mut db_counter, random_key)
                    .unwrap(),
                None,
            );
            assert_eq!(
                cache
                    .nearest_upper_preimage_mut(&mut db_counter, random_key)
                    .unwrap(),
                None,
            );
        }

        // Init uses 2 queries. After that, min > max so all queries short-circuit.
        assert_eq!(db_counter.accesses(), 2);
    }

    #[test]
    fn test_approx_cache_hits_on_known_images() {
        let mut db = MemoryPreimagesProvider::new();

        let mut images = Vec::new();
        for _ in 0..10 {
            images.push(db.insert(Preimage::copy_from_slice(&Image::random().0)));
        }

        let mut db_counter = CounterPreimagesProviderMut::new(&db);
        let mut cache = ApproxCache::<16>::new_init(&mut db_counter, ()).unwrap();

        // First pass: query at exact preimage images to populate the cache.
        // These are guaranteed to share a prefix with the result, so the cache
        // stores entries under the same key that future lookups will use.
        for &image in &images {
            cache
                .nearest_lower_preimage_mut(&mut db_counter, image)
                .unwrap();
        }

        let accesses_after_first_pass = db_counter.accesses();

        // Second pass with the same images: all cache hits.
        for &image in &images {
            cache
                .nearest_lower_preimage_mut(&mut db_counter, image)
                .unwrap();
        }

        assert_eq!(db_counter.accesses(), accesses_after_first_pass);
    }

    #[test]
    fn test_approx_cache_below_min_above_max() {
        let mut db = MemoryPreimagesProvider::new();

        // Insert a single preimage so min == max == its image.
        let preimage = Preimage::copy_from_slice(&Image::random().0);
        let image = db.insert(preimage);

        let mut db_counter = CounterPreimagesProviderMut::new(&db);
        let mut cache = ApproxCache::<16>::new_init(&mut db_counter, ()).unwrap();
        let accesses_after_init = db_counter.accesses();

        // Queries below min should return None without hitting the provider.
        if image > B256::ZERO {
            let below = B256::ZERO;
            assert_eq!(
                cache
                    .nearest_lower_preimage_mut(&mut db_counter, below)
                    .unwrap(),
                None,
            );
        }

        // Queries above max should return None without hitting the provider.
        if image < B256_MAX {
            let above = B256_MAX;
            assert_eq!(
                cache
                    .nearest_upper_preimage_mut(&mut db_counter, above)
                    .unwrap(),
                None,
            );
        }

        // No additional provider accesses for the short-circuited queries.
        assert_eq!(db_counter.accesses(), accesses_after_init);
    }
}
