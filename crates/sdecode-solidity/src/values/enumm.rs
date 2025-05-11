use std::marker::PhantomData;

use quick_impl::quick_impl;
use sdecode_core::StorageReader;

use crate::{SolStorageType, data_types};

use super::{SolLayoutError, SolStorageValue};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[quick_impl]
pub struct SolEnumHelper<E, SolE>(
    #[quick_impl(impl Deref, impl DerefMut, impl From)] pub E,
    PhantomData<SolE>,
);

impl<E, SolE> SolEnumHelper<E, SolE> {
    pub const fn new(value: E) -> Self {
        Self(value, PhantomData)
    }
}

impl<E, SolE> SolStorageValue<SolE> for SolEnumHelper<E, SolE>
where
    E: TryFrom<u8>,
    SolE: SolStorageType,
{
    fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
    where
        Reader: StorageReader,
    {
        let discr = <u8 as SolStorageValue<data_types::Uint<8>>>::decode_storage(storage_reader)?;
        let res = E::try_from(discr).map_err(|_| SolLayoutError::Err)?;
        Ok(Self::new(res))
    }
}
