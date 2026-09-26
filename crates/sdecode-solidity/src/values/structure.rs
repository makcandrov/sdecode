use quick_impl::quick_impl;
use sdecode_core::StorageReader;
use std::{hash::Hash, marker::PhantomData};

use crate::{SolLayoutError, SolStorageType, SolStorageValue};

/// Solidity doesn't have tuples, so it wouldn't be correct to implement `SolStorageTypeValue` on
/// tuples. Instead, it is implemented on a wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[quick_impl]
pub struct SolStructureHelper<SolStructure, Tuple, SolTuple>(
    #[quick_impl(impl Deref, impl DerefMut)] pub Tuple,
    PhantomData<(SolStructure, SolTuple)>,
);

impl<SolStructure, Tuple, SolTuple> SolStructureHelper<SolStructure, Tuple, SolTuple> {
    pub const fn new(tuple: Tuple) -> Self {
        Self(tuple, PhantomData)
    }
}

macro_rules! impl_sol_storage_type_structure {
    ($( $ty:ident $sol:ident ),*) => {
        impl<SolStructure, $($ty, $sol,)*> SolStorageValue<SolStructure> for SolStructureHelper<SolStructure, ($($ty,)*), ($($sol,)*)>
        where
            SolStructure: SolStorageType,
            $(
                $ty: SolStorageValue<$sol>,
                $sol: SolStorageType,
            )*
        {
            fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
            where
                Reader: StorageReader
            {
                Ok(Self::new(($(
                    $ty::decode_storage(storage_reader)?,
                )*)))
            }
        }
    };
}

macro_rules! impl_sol_storage_type_structures {
    ([$( $done_ty:ident $done_sol:ident )*]) => {};
    ([$( $done_ty:ident $done_sol:ident )*] $ty:ident $sol:ident $($rest:tt)*) => {
        impl_sol_storage_type_structure!($( $done_ty $done_sol, )* $ty $sol);
        impl_sol_storage_type_structures!([$( $done_ty $done_sol )* $ty $sol] $($rest)*);
    };
}

impl<SolStructure, A, SolA> SolStorageValue<SolStructure>
    for SolStructureHelper<SolStructure, A, SolA>
where
    SolStructure: SolStorageType,
    A: SolStorageValue<SolA>,
    SolA: SolStorageType,
{
    fn decode_storage<Reader>(storage_reader: &mut Reader) -> Result<Self, SolLayoutError>
    where
        Reader: StorageReader,
    {
        Ok(Self::new(A::decode_storage(storage_reader)?))
    }
}

impl_sol_storage_type_structures!(
    []
    A SolA
    B SolB
    C SolC
    D SolD
    E SolE
    F SolF
    G SolG
    H SolH
    I SolI
    J SolJ
    K SolK
    L SolL
    M SolM
    N SolN
    O SolO
    P SolP
    Q SolQ
    R SolR
    S SolS
    T SolT
    U SolU
    V SolV
    W SolW
    X SolX
    Y SolY
    Z SolZ
);
