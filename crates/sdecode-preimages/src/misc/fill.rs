use quick_impl::quick_impl;

use crate::{
    Image, MemoryPreimagesProvider, PreimageEntry, PreimagesProvider, PreimagesProviderMut,
    WrapPreimagesProvider,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[quick_impl]
pub struct PreimagesProviderFiller<P> {
    #[quick_impl(pub get = "inner_{}", pub get_mut = "inner_{}_mut", pub into = "into_inner_{}")]
    provider: P,

    #[quick_impl(pub get = "{}", pub into)]
    result: MemoryPreimagesProvider,
}
impl<P: PreimagesProvider> PreimagesProviderFiller<WrapPreimagesProvider<P>> {
    pub const fn new(preimages_provider: P) -> Self {
        Self::new_mut(WrapPreimagesProvider(preimages_provider))
    }
}

impl<P: PreimagesProviderMut> PreimagesProviderFiller<P> {
    pub const fn new_mut(preimages_provider: P) -> Self {
        Self {
            provider: preimages_provider,
            result: MemoryPreimagesProvider::new(),
        }
    }
}

impl<P: PreimagesProviderMut> PreimagesProviderMut for PreimagesProviderFiller<P> {
    type Error = P::Error;

    fn nearest_lower_preimage_mut(
        &mut self,
        image: Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        let entry = self.provider.nearest_lower_preimage_mut(image)?;
        if let Some(entry) = &entry {
            self.result
                .insert_unchecked_with(entry.image(), || entry.preimage().clone());
        }
        Ok(entry)
    }

    fn nearest_upper_preimage_mut(
        &mut self,
        image: Image,
    ) -> Result<Option<PreimageEntry>, Self::Error> {
        let entry = self.provider.nearest_upper_preimage_mut(image)?;
        if let Some(entry) = &entry {
            self.result
                .insert_unchecked_with(entry.image(), || entry.preimage().clone());
        }
        Ok(entry)
    }
}
