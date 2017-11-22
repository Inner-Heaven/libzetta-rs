
/// Everything you need to work with zpools. Since there is no public library
/// to work with zpool —
/// the default impl will call to `zpool(8)`.
use std::io;
use std::iter::Map;
pub mod vdev;
pub use self::vdev::{Disk, Vdev};

pub mod topology;
pub use self::topology::{Topology, TopologyBuilder};

pub mod open3;
pub use self::open3::ZpoolOpen3;

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

pub type ZpoolProperties = Map<ZpoolProperty, String>;
/// List of available properties.
pub enum ZpoolProperty {
    NotYet
}

/// Generic interface to manage zpools. End goal is to cover most of `zpool(8)`.
/// Highly unlikely to reach that goal functions will be added as project grows.
/// Using trait here, so I can mock it in unit tests.
pub trait ZpoolEngine {
    /// Check if pool with given name exists. This will return error only if
    /// call to `zpool` fail.
    fn exists<N: AsRef<str>>(&self, name: N) -> ZpoolResult<bool>;
    /// Create new zpool.
    fn create<N: AsRef<str>>(&self, name: N, topology: Topology, properties: Option<ZpoolProperties>) -> ZpoolResult<()>;
    /// Destroy zpool
    fn destroy<N: AsRef<str>>(&self, name: N, force: bool) -> ZpoolResult<()>;
    //fn get_properties<N: AsRef<str>>(&self, name: N)
}
