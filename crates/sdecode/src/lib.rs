#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![doc = include_str!("../../../README.md")]

pub use sdecode_core as core;
pub use sdecode_preimages as preimages;

pub mod solidity {
    pub use sdecode_solidity::{
        SolLayoutError, SolMappingKeyType, SolMappingKeyValue, SolStorageType, SolStorageValue,
        SolWordType, helpers, sol_types,
    };

    #[doc(inline)]
    pub use super::sol_storage;
}

pub use core::{StorageDecode, StorageEntries, StorageError};

pub use preimages::{Image, Preimage, PreimageEntry, PreimagesProvider, PreimagesProviderMut};

#[doc(hidden)]
#[macro_export]
macro_rules! sol_storage {
    ($($t:tt)*) => {
        $crate::__private::sdecode_solidity_macro::sol_storage! {
            #![sdecode(reexport = $crate::__private::sdecode_solidity)]
            $($t)*
        }
    };
}

#[doc(hidden)]
pub mod __private {
    #[doc(hidden)]
    pub use sdecode_solidity;

    #[doc(hidden)]
    pub use sdecode_solidity_macro;
}
