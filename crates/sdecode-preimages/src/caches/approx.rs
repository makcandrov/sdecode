use alloy_primitives::{B256, FixedBytes};
use hashbrown::HashMap;

use crate::{PreimageEntry, PreimagesProviderMut, utils::B256_MAX};

use super::PreimagesCache;

#[derive(Debug, Clone)]
pub struct ApproxCache<const N: usize> {
    cache: HashMap<FixedBytes<N>, PreimageEntry>,
    min: B256,
    max: B256,
}

impl<const N: usize, P: PreimagesProviderMut> PreimagesCache<P> for ApproxCache<N> {
    fn new(provider: &mut P) -> Result<Self, P::Error> {
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

    fn nearest_lower_preimage_mut(
        &mut self,
        provider: &mut P,
        image: crate::Image,
    ) -> Result<Option<PreimageEntry>, <P as PreimagesProviderMut>::Error> {
        if image < self.min {
            return Ok(None);
        }
        if let Some(entry) = self.cache.get(&image[..N]) {
            assert!(entry.image() <= image, "collision, chosen N is too small");
            Ok(Some(entry.clone()))
        } else {
            let entry = provider
                .nearest_lower_preimage_mut(image)?
                .expect("image is greater than `min` then a preimage must exist");
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
            assert!(entry.image() >= image, "collision, chosen N is too small");
            Ok(Some(entry.clone()))
        } else {
            let entry = provider
                .nearest_lower_preimage_mut(image)?
                .expect("image is lesser than `max` then a preimage must exist");
            let key = FixedBytes::<N>::from_slice(&entry.image()[..N]);
            self.cache.insert(key, entry.clone());
            Ok(Some(entry))
        }
    }
}
