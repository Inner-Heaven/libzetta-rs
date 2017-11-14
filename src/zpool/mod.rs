/// Everything you need to work with zpools. Since there is no public library to work with zpool —
/// the default impl will call to `zpool(8)`.

use std::path::Path;

quick_error! {
    #[derive(Debug)]
    /// Error kinds. This type will be used across zpool module.
    pub enum ZpoolError {
        /// zpool executable not found in path.
        CommandNotFound {},
        /// Trying to manipulate non-existant pool.
        PoolNotFound {},
        /// At least one vdev points to incorrect location.
        /// If vdev type is File then it means file not found.
        DeviceNotFound{}
    }
}

/// Type alias to `Result<T, ZpoolError>`.
pub type ZpoolResult<T> = Result<T, ZpoolError>

/// Structure representing zpool.
/// It holds very little information about zpool itself besides it's name. Only gurantee this type
/// provide is that at some point of time zpool with such name existed when structure was
/// instanciated.
///
/// It doesn't hold any properties and only hold stats like capacity and health status at the point
/// of structure initilization.
pub struct Zpool {
    name: String
}

/// Basic building block of Zpool is [Vdev](https://www.freebsd.org/doc/handbook/zfs-term.html).
/// One Vdev can hold multiple Vdevs of Vdevs.
/// For example `Vdev::Mirror(vec![Vdev::File(foo), Vdev:Bare(bar)])` is totally fine.
pub enum Vdev {
    /// Use empty file as block storage. Useful for testing. Provide path to the file. File must
    /// exist.
    File(Path)
    /// Just one disk. Either whole disk identifier or slice.
    Bare(String),
    /// A mirror of multiple vdevs
    Mirror(Vec<Vdev>),
    /// ZFS implements [RAID-Z](https://blogs.oracle.com/ahl/what-is-raid-z), a variation on standard RAID-5 that offers better distribution of
    /// parity and eliminates the “RAID-5 write hole”.
    RaidZ(Vec<Vdev>),
    /// The same as RAID-Z, but with 2 parity drives.
    RaidZ2(Vec<Vdev>),
    /// The same as RAID-Z, but with 3 parity drives.
    RaidZ3(Vec<Vdev>),

}

/// Generic interface to manage zpools. End goal is to cover most of `zpool(8)`.
/// Highly unlikely to reach that goal functions will be added as project grows.
/// Using trait here, so I can mock it in unit tests.
pub trait ZpoolEngine {
    /// Check if pool with given name exists. This will return error only if call to `zpool` fail.
    fn exists(&self, name: &str) -> ZpoolResult<bool>;
    fn create(&self, name: &str, vdev: Vdev) -> ZpoolResult<Zpool>;
    fn get(&self, name: &str) -> ZpoolResult<Zpool>;
}
