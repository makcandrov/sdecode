use std::collections::BTreeMap;

use alloy_primitives::U256;
use quick_impl::quick_impl;

use crate::{Image, PreimageEntry, PreimagesProvider, PreimagesProviderMut, WrapPreimagesProvider};

/// Cache for general purpose use.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[quick_impl]
pub struct GeneralPreimagesCache<P> {
    #[quick_impl(get = "{}", get_mut = "{}_mut", into)]
    provider: P,
    cache: BTreeMap<U256, Option<PreimageEntry>>,
}

impl<P: PreimagesProvider> GeneralPreimagesCache<WrapPreimagesProvider<P>> {
    pub const fn new(preimages_provider: P) -> Self {
        Self::new_mut(WrapPreimagesProvider(preimages_provider))
    }
}

impl<P: PreimagesProviderMut> GeneralPreimagesCache<P> {
    pub const fn new_mut(preimages_provider: P) -> Self {
        Self {
            provider: preimages_provider,
            cache: BTreeMap::new(),
        }
    }
}

impl<P: PreimagesProviderMut> PreimagesProviderMut for GeneralPreimagesCache<P> {
    type Error = P::Error;

    fn nearest_lower_preimage_mut(
        &mut self,
        image: Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        self.provider.nearest_lower_preimage_mut(image)
    }

    fn nearest_upper_preimage_mut(
        &mut self,
        image: Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        self.provider.nearest_upper_preimage_mut(image)
    }
}
