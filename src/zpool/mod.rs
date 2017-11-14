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
pub type ZpoolResult<T> = Result<T, ZpoolError>;

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

/// Every vdev can be backed either by block device or sparse file.
pub enum Disk {
    /// Sparse file based device. 
    File(PathBuf),
    /// Block device.
    Disk(PathBuf),
}

impl Disk {
    /// Verify that disk is valid. Just because it valid doesn't mean zpool can use it.
    /// all it does it verifies that path exists. For now they both look the same. Distinction exists to make sure it will work in the future.
    pub fn is_valid(&self) -> bool {
        match *self {
            File(path) => path.exists(),
            Disk(path) => path.exists().
        }
    }
    pub fn into_arg(self) -> OsString {
        match *self {
            File(path) => path.into_os_string(),
            Disk(path) => path.into_os_string().
        }
    }
}

/// Basic building block of [Zpool](https://www.freebsd.org/doc/handbook/zfs-term.html).
pub enum Vdev {
    /// Just a single disk or file.
    Naked(Disk)
    /// A mirror of multiple vdevs
    Mirror(Vec<Disk>),
    /// ZFS implements [RAID-Z](https://blogs.oracle.com/ahl/what-is-raid-z), a variation on standard RAID-5 that offers better distribution of
    /// parity and eliminates the “RAID-5 write hole”.
    RaidZ(Vec<Disk>),
    /// The same as RAID-Z, but with 2 parity drives.
    RaidZ2(Vec<Disk>),
    /// The same as RAID-Z, but with 3 parity drives.
    RaidZ3(Vec<Disk>),
}

impl Vdev {
    #[inline(always)]
    fn is_valid_raid(disks: &Vec<Disk>, min_disks: u8) {
        if disks.len() < min_disks {
            return false;
        }
        disks.iter().all(Disk::is_valid)
    }
    /// Check if given Vdev is valid.
    /// 
    /// For Naked it means that what ever it points to exists.
    /// 
    /// For Mirror it checks that it's atleast two valid disks.
    /// 
    /// For RaidZ it checks that it's aleast three valid disk. And so goes on.
    pub fn is_valid(&self) -> bool {
        match *self {
            Naked(ref disk) => disk.is_valid(),
            Mirror(ref disks) => is_valid_raid(disks, 2),
            RaidZ(ref disks) => is_valid_raid(disks, 2),
            RaidZ2(ref disks) => is_valid_raid(disks, 3),
            RaidZ3(ref disks) => is_valid_raid(disks, 4),
        }
    }

    #[inline(always)]
    fn into_args<T: Into<OsString>>(vdev_type: T, disks: Vec<Disk>) -> Vec<OsString> {
        let ret = Vec::with_capacity(disks.len() + 1);
        ret.push(vdev_type.into());
        for disk in disks.into_iter() {
            ret.push(disk.into_arg());
        }
        return ret;
    }
    /// Make turn Vdev into list of arguments. 
    pub fn to_args(self) -> Vec<OsString> {
        match *self {
            Named(disk) => vec![disk.into_arg()],
            Mirror(disks) => into_args("mirror", disks),
            RaidZ(disks) => into_args("raidz", disks),
            RaidZ2(disks) => into_args("raidz2", disks),
            RaidZ3(disks) => into_args("raidz3", disks)
        }
    }
}

/// Generic interface to manage zpools. End goal is to cover most of `zpool(8)`.
/// Highly unlikely to reach that goal functions will be added as project grows.
/// Using trait here, so I can mock it in unit tests.
pub trait ZpoolEngine {
    /// Check if pool with given name exists. This will return error only if call to `zpool` fail.
    fn exists(&self, name: &str) -> ZpoolResult<bool>;
    fn create(&self, name: &str, vdev: Vec<Vdev>) -> ZpoolResult<Zpool>;
    fn get(&self, name: &str) -> ZpoolResult<Zpool>;
}
