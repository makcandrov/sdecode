use super::{SolLayoutError, SolStorageValue};
use crate::SolStorageType;
use paste::paste;
use quick_impl::quick_impl;
use sdecode_core::StorageReader;
use std::{hash::Hash, marker::PhantomData};

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
    ($( $ty: ident),*) => {
        paste! {
            impl<SolStructure, $($ty, [<Sol $ty>],)*> SolStorageValue<SolStructure> for SolStructureHelper<SolStructure, ($($ty,)*), ($([<Sol $ty>],)*)>
            where
                SolStructure: SolStorageType,
                $(
                    $ty: SolStorageValue<[<Sol $ty>]>,
                    [<Sol $ty>]: SolStorageType,
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
        }
    }
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

impl_sol_storage_type_structure!(A);
impl_sol_storage_type_structure!(A, B);
impl_sol_storage_type_structure!(A, B, C);
impl_sol_storage_type_structure!(A, B, C, D);
impl_sol_storage_type_structure!(A, B, C, D, E);
impl_sol_storage_type_structure!(A, B, C, D, E, F);
impl_sol_storage_type_structure!(A, B, C, D, E, F, G);
impl_sol_storage_type_structure!(A, B, C, D, E, F, G, H);
impl_sol_storage_type_structure!(A, B, C, D, E, F, G, H, I);
impl_sol_storage_type_structure!(A, B, C, D, E, F, G, H, I, J);
impl_sol_storage_type_structure!(A, B, C, D, E, F, G, H, I, J, K);
impl_sol_storage_type_structure!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_sol_storage_type_structure!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_sol_storage_type_structure!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_sol_storage_type_structure!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_sol_storage_type_structure!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_sol_storage_type_structure!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_sol_storage_type_structure!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_sol_storage_type_structure!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_sol_storage_type_structure!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_sol_storage_type_structure!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U
);
impl_sol_storage_type_structure!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V
);
impl_sol_storage_type_structure!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W
);
impl_sol_storage_type_structure!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X
);
impl_sol_storage_type_structure!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y
);
impl_sol_storage_type_structure!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);
