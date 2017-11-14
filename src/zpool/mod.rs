
/// Everything you need to work with zpools. Since there is no public library
/// to work with zpool —
/// the default impl will call to `zpool(8)`.

pub mod vdev;
pub use self::vdev::{Disk, Vdev};

quick_error! {
    /// Error kinds. This type will be used across zpool module.
    #[derive(Debug)]
    pub enum ZpoolError {
        /// zpool executable not found in path.
        CommandNotFound {}
        /// Trying to manipulate non-existant pool.
        PoolNotFound {}
        /// At least one vdev points to incorrect location.
        /// If vdev type is File then it means file not found.
        DeviceNotFound {}
    }
}

/// Type alias to `Result<T, ZpoolError>`.
pub type ZpoolResult<T> = Result<T, ZpoolError>;

/// Structure representing zpool.
/// It holds very little information about zpool itself besides it's name. Only
/// gurantee this type
/// provide is that at some point of time zpool with such name existed when
/// structure was
/// instanciated.
///
/// It doesn't hold any properties and only hold stats like capacity and health
/// status at the point
/// of structure initilization.
#[derive(Debug, Clone)]
pub struct Zpool {
    name: String,
}


/// Generic interface to manage zpools. End goal is to cover most of `zpool(8)`.
/// Highly unlikely to reach that goal functions will be added as project grows.
/// Using trait here, so I can mock it in unit tests.
pub trait ZpoolEngine {
    /// Check if pool with given name exists. This will return error only if
    /// call to `zpool` fail.
    fn exists(&self, name: &str) -> ZpoolResult<bool>;
    fn create(&self, name: &str, vdev: Vec<Vdev>) -> ZpoolResult<Zpool>;
    fn get(&self, name: &str) -> ZpoolResult<Zpool>;
}
