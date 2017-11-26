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

use regex::Regex;


lazy_static! {
    static ref RE_REUSE_VDEV: Regex = Regex::new(r"following errors:\n(\S+) is part of active pool '(\S+)'").expect("failed to compile re_vdev_reuse)");
}

quick_error! {
    /// Error kinds. This type will be used across zpool module.
    #[derive(Debug)]
    pub enum ZpoolError {
        /// `zpool` not found in path. Open3 specific error.
        CmdNotFound {}
        /// zpool executable not found in path.
        Io(err: io::Error) {
            cause(err)
        }
        /// Trying to manipulate non-existant pool.
        PoolNotFound {}
        /// Givin topolog failed validation.
        InvalidTopology {}
        /// Trying to create new Zpool, but one or more vdevs already used in another pool.
        VdevReuse(vdev: String, pool: String) {
            display("{} is part of {}", vdev, pool)
        }
        /// Don't know (yet) how to categorize this error. If you see this error - open an issues.
        Other(err: String) {}
    }
}

impl ZpoolError {
    pub fn kind(&self) -> ZpoolErrorKind {
        match *self {
            ZpoolError::CmdNotFound => ZpoolErrorKind::CmdNotFound,
            ZpoolError::Io(_)       => ZpoolErrorKind::Io,
            ZpoolError::PoolNotFound    => ZpoolErrorKind::PoolNotFound,
            ZpoolError::InvalidTopology  => ZpoolErrorKind::InvalidTopology,
            ZpoolError::VdevReuse(_,_)  => ZpoolErrorKind::VdevReuse,
            ZpoolError::Other(_)        => ZpoolErrorKind::Other,
        }
    }
}

/// This is a hack to allow error identification without 100500 lines of code because
/// `std::io::Error` doesn't implement `PartialEq`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ZpoolErrorKind {
        /// `zpool` not found in path. Open3 specific error.
        CmdNotFound,
        Io,
        /// Trying to manipulate non-existant pool.
        PoolNotFound,
        /// At least one vdev points to incorrect location.
        /// If vdev type is File then it means file not found.
        DeviceNotFound,
        /// Trying to create new Zpool, but one or more vdevs already used in another pool.
        VdevReuse,
        /// Givin topolog failed validation.
        InvalidTopology,
        /// Don't know (yet) how to categorize this error. If you see this error - open an issues.
        Other,
}

impl From<io::Error> for ZpoolError {
    fn from(err: io::Error) -> ZpoolError {
        match err.kind() {
            io::ErrorKind::NotFound => ZpoolError::CmdNotFound,
            _                       => ZpoolError::Io(err)
        }
    }
}

impl ZpoolError {
    /// Try to convert stderr into internal error type.
    pub fn from_stderr(stderr_raw: &[u8]) -> ZpoolError {
        let stderr = String::from_utf8_lossy(stderr_raw);
        if RE_REUSE_VDEV.is_match(&stderr) {
            let caps = RE_REUSE_VDEV.captures(&stderr).unwrap();
            ZpoolError::VdevReuse(caps.get(1).unwrap().as_str().into(), caps.get(2).unwrap().as_str().into())
        } else {
            ZpoolError::Other(stderr.into())
        }
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
    /// Version of create that doesn't check validness of topology or options.
    fn create_unchecked<N: AsRef<str>>(&self, name: N, topology: Topology) -> ZpoolResult<()>;
    /// Create new zpool.
    fn create<N: AsRef<str>>(&self, name: N, topology: Topology) -> ZpoolResult<()> {
        if !topology.is_suitable_for_create() {
            return Err(ZpoolError::InvalidTopology);
        }
        self.create_unchecked(name, topology)
    }
    /// Version of destroy that doesn't verify if pool exists before removing it.
    fn destroy_unchecked<N: AsRef<str>>(&self, name: N, force: bool) -> ZpoolResult<()>;
    /// Destroy zpool.
    fn destroy<N: AsRef<str>>(&self, name: N, force: bool) -> ZpoolResult<()> {
        if !self.exists(&name)? {
            return Err(ZpoolError::PoolNotFound);
        }

        self.destroy_unchecked(name, force)
    }
    // fn get_properties<N: AsRef<str>>(&self, name: N)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn error_parsing() {
        let vdev_reuse_text = b"invalid vdev specification\nuse '-f' to override the following errors:\n/vdevs/vdev0 is part of active pool 'tank'";
        let unknown_text = b"wat";

        let err = ZpoolError::from_stderr(vdev_reuse_text);

        assert_eq!(ZpoolErrorKind::VdevReuse, err.kind());
        if let ZpoolError::VdevReuse(vdev, pool) = err {
            assert_eq!("/vdevs/vdev0", vdev);
            assert_eq!("tank", pool);
        }

        let err = ZpoolError::from_stderr(unknown_text);
        assert_eq!(ZpoolErrorKind::Other, err.kind());
        if let ZpoolError::Other(text) = err {
            assert_eq!("wat", text);
        }
    }

    #[test]
    fn io_error_from() {
        let cmd_not_found = io::Error::new(io::ErrorKind::NotFound, "oh no");
        let err = ZpoolError::from(cmd_not_found);
        assert_eq!(ZpoolErrorKind::CmdNotFound, err.kind());

        let other = io::Error::new(io::ErrorKind::Other, "oh now");
        let err = ZpoolError::from(other);
        assert_eq!(ZpoolErrorKind::Io, err.kind());
    }
}
