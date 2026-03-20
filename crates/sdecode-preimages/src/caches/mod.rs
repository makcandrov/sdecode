mod approx;
pub use approx::{ApproxCache, ApproxCachedProvider};

mod general;
pub use general::{GeneralCache, GeneralCachedProvider};

mod storage;
pub use storage::StoragePreimagesCache;
