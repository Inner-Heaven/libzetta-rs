/// Everything you need to work with zpools. Since there is no public library
/// to work with zpool —
/// the default impl will call to `zpool(8)`.
use std::io;
pub mod vdev;
pub use self::vdev::{Disk, Vdev};

pub mod topology;
pub use self::topology::{Topology, TopologyBuilder};

pub mod open3;
pub use self::open3::ZpoolOpen3;

pub mod properties;
pub use self::properties::{CacheType, FailMode, Health, ZpoolProperties, ZpoolPropertiesWrite,
                           ZpoolPropertiesWriteBuilder};

quick_error! {
    /// Error kinds. This type will be used across zpool module.
    #[derive(Debug)]
    pub enum ZpoolError {
        /// zpool executable not found in path.
        Io(err: io::Error) {
            from()
            cause(err)
        }
        /// Trying to manipulate non-existant pool.
        PoolNotFound {}
        /// At least one vdev points to incorrect location.
        /// If vdev type is File then it means file not found.
        DeviceNotFound {}
    }
}

/// Type alias to `Result<T, ZpoolError>`.
pub type ZpoolResult<T> = Result<T, ZpoolError>;


/// Generic interface to manage zpools. End goal is to cover most of `zpool(8)`.
/// Using trait here, so I can mock it in unit tests.
pub trait ZpoolEngine {
    /// Check if pool with given name exists. This should not return
    /// [`ZpoolError::PoolNotFound`](enum.ZpoolError.html) error, instead
    /// it should return `Ok(false)`.
    fn exists<N: AsRef<str>>(&self, name: N) -> ZpoolResult<bool>;
    /// Create new zpool.
    fn create<N: AsRef<str>>(&self, name: N, topology: Topology) -> ZpoolResult<()>;
    /// Destroy zpool
    fn destroy<N: AsRef<str>>(&self, name: N, force: bool) -> ZpoolResult<()>;
    // fn get_properties<N: AsRef<str>>(&self, name: N)
}
