mod approx;
pub use approx::{ApproxCache, ApproxCachedProvider, APPROX_CACHE_DEFAULT_PREFIX_LEN};

mod general;
pub use general::{GeneralCache, GeneralCachedProvider};

mod storage;
pub use storage::StoragePreimagesCache;
