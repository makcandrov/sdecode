#![cfg_attr(not(test), warn(unused_crate_dependencies))]

mod types;
pub use types::{SolMappingKeyType, SolStorageType};

pub mod data_types {
    use super::*;

    pub use types::{
        Address, Array, Bool, ByteCount, Bytes, FixedArray, FixedBytes, Function, Int, IntBitCount,
        Mapping, String, SupportedFixedBytes, SupportedInt, Uint,
    };
}

mod values;
pub use values::{SolLayoutError, SolMappingKeyValue, SolStorageValue, SolWordType, helpers};

mod unknown;

mod utils;

#[macro_export]
macro_rules! sol_storage {
    ($($t:tt)*) => {
        $crate::__private::sdecode_solidity_macro::sol_storage! {
            #![sdecode(reexport = $crate)]
            $($t)*
        }
    };
}

#[doc(hidden)]
pub mod __private {
    #[doc(hidden)]
    pub use alloy_primitives;

    #[doc(hidden)]
    pub use sdecode_core;

    #[doc(hidden)]
    pub use sdecode_preimages;

    #[doc(hidden)]
    pub use sdecode_solidity_macro;
}
