mod approx;
pub use approx::{APPROX_CACHE_DEFAULT_PREFIX_LEN, ApproxCache, ApproxCachedProvider};

mod general;
pub use general::{GeneralCache, GeneralCachedProvider};

mod storage;
pub use storage::StoragePreimagesCache;
