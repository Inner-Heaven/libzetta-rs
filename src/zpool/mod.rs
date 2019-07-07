/// Everything you need to work with zpools. Since there is no public library
/// to work with zpool —
/// the default impl will call to `zpool(8)`.
use std::io;
use std::{default::Default,
          ffi::OsStr,
          num::{ParseFloatError, ParseIntError},
          path::PathBuf};

use regex::Regex;

pub use self::{description::{Reason, Zpool},
               open3::ZpoolOpen3,
               properties::{CacheType, FailMode, Health, PropPair, ZpoolProperties,
                            ZpoolPropertiesWrite, ZpoolPropertiesWriteBuilder},
               topology::{CreateZpoolRequest, CreateZpoolRequestBuilder},
               vdev::{CreateVdevRequest, Disk, Vdev, VdevType}};

pub mod open3;
pub mod properties;
pub mod topology;
pub mod vdev;

pub mod description;
lazy_static! {
    static ref RE_REUSE_VDEV_ZOL: Regex = Regex::new(r"cannot create \S+: one or more vdevs refer to the same device, or one of\nthe devices is part of an active md or lvm device\n").expect("failed to compile RE_VDEV_REUSE_ZOL)");
    static ref RE_REUSE_VDEV: Regex = Regex::new(r"following errors:\n(\S+) is part of active pool '(\S+)'").expect("failed to compile RE_VDEV_REUSE)");
    static ref RE_REUSE_VDEV2: Regex = Regex::new(r"invalid vdev specification\nuse '-f' to override the following errors:\n(\S+) is part of potentially active pool '(\S+)'\n?").expect("failed to compile RE_VDEV_REUSE2)");
    static ref RE_TOO_SMALL: Regex = Regex::new(r"cannot create \S+: one or more devices is less than the minimum size \S+").expect("failed to compile RE_TOO_SMALL");
    static ref RE_PERMISSION_DENIED: Regex = Regex::new(r"cannot create \S+: permission denied\n").expect("failed to compile RE_PERMISSION_DENIED");
    static ref RE_NO_ACTIVE_SCRUBS: Regex = Regex::new(r"cannot (pause|cancel) scrubbing .+: there is no active scrub\n").expect("failed to compile RE_NO_ACTIVE_SCRUBS");
    static ref RE_NO_SUCH_POOL: Regex = Regex::new(r"cannot open '\S+': no such pool\n?").expect("failed to compile RE_NO_SUCH_POOL");
    static ref RE_NO_VALID_REPLICAS: Regex = Regex::new(r"cannot offline \S+: no valid replicas\n?").expect("failed to compile RE_NO_VALID_REPLICAS");
    static ref RE_CANNOT_ATTACH: Regex = Regex::new(r"cannot attach \S+ to \S+ can only attach to mirrors and top-level disks").expect("failed to compile RE_CANNOT_ATTACH");
    static ref RE_NO_SUCH_DEVICE: Regex = Regex::new(r"cannot attach \S+ to \S+: no such device in pool").expect("failed to compile RE_NO_SUCH_DEVICE");
    static ref RE_ONLY_DEVICE: Regex = Regex::new(r"cannot detach \S+ only applicable to mirror and replacing vdevs").expect("failed to compile RE_ONLY_DEVICE");
    static ref RE_MISMATCH_REPLICATION: Regex = Regex::new(r"invalid vdev specification\nuse '-f' to override the following errors:\nmismatched replication level:.+").expect("failed to compile RE_MISMATCHED_REPLICATION");
    static ref RE_INVALID_CACHE_DEVICE: Regex = Regex::new(r"cannot add to \S+: cache device must be a disk or disk slice\n?").expect("failed to compile RE_INVALID_CACHE_DEVICE");
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
        /// Trying to manipulate non-existent pool.
        PoolNotFound {}
        /// Given topology failed validation.
        InvalidTopology {}
        /// Trying to create new Zpool, but one or more vdevs already used in another pool.
        VdevReuse(vdev: String, pool: String) {
            display("{} is part of {}", vdev, pool)
        }
        /// Failed to parse value. Ideally you never see it, if you see it - it's a bug.
        ParseError {
            from(ParseIntError)
            from(ParseFloatError)
        }
        /// Device used in CreateZpoolRequest is smaller than 64M
        DeviceTooSmall {}
        /// Permission denied to create zpool. This might happened because:
        /// a) you running it as not root
        /// b) you running it inside jail that isn't allowed to operate zfs
        PermissionDenied {}
        /// Trying to pause/stop scrub thas either never stared or already completed
        NoActiveScrubs {}
        /// Trying to take only device offline.
        NoValidReplicas {}
        /// Couldn't parse string to raid type.
        UnknownRaidType(source: String) {}
        /// Cannot attach a device to device that is part of raidz. It can only be attached to mirrors and top-level disks.
        CannotAttach {}
        /// Operation on device that was not found in the pool.
        NoSuchDevice {}
        /// Trying to detach a device from vdev without any valid replicas left.
        OnlyDevice {}
        /// Trying to add vdev with wring replication level to existing zpool with different replication level.
        /// For example: mirror to zpool.
        MismatchedReplicationLevel {}
        /// Cache device must a disk or disk slice.
        InvalidCacheDevice {}
        /// Don't know (yet) how to categorize this error. If you see this error - open an issues.
        Other(err: String) {}
    }
}

impl ZpoolError {
    pub fn kind(&self) -> ZpoolErrorKind {
        match *self {
            ZpoolError::CmdNotFound => ZpoolErrorKind::CmdNotFound,
            ZpoolError::Io(_) => ZpoolErrorKind::Io,
            ZpoolError::PoolNotFound => ZpoolErrorKind::PoolNotFound,
            ZpoolError::InvalidTopology => ZpoolErrorKind::InvalidTopology,
            ZpoolError::VdevReuse(..) => ZpoolErrorKind::VdevReuse,
            ZpoolError::ParseError => ZpoolErrorKind::ParseError,
            ZpoolError::DeviceTooSmall => ZpoolErrorKind::DeviceTooSmall,
            ZpoolError::PermissionDenied => ZpoolErrorKind::PermissionDenied,
            ZpoolError::NoActiveScrubs => ZpoolErrorKind::NoActiveScrubs,
            ZpoolError::NoValidReplicas => ZpoolErrorKind::NoValidReplicas,
            ZpoolError::UnknownRaidType(_) => ZpoolErrorKind::UnknownRaidType,
            ZpoolError::CannotAttach => ZpoolErrorKind::CannotAttach,
            ZpoolError::NoSuchDevice => ZpoolErrorKind::NoSuchDevice,
            ZpoolError::OnlyDevice => ZpoolErrorKind::OnlyDevice,
            ZpoolError::MismatchedReplicationLevel => ZpoolErrorKind::MismatchedReplicationLevel,
            ZpoolError::InvalidCacheDevice => ZpoolErrorKind::InvalidCacheDevice,
            ZpoolError::Other(_) => ZpoolErrorKind::Other,
        }
    }
}

/// This is a hack to allow error identification without 100500 lines of code
/// because
/// `std::io::Error` doesn't implement `PartialEq`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ZpoolErrorKind {
    /// `zpool` not found in path. Open3 specific error.
    CmdNotFound,
    Io,
    /// Trying to manipulate non-existent pool.
    PoolNotFound,
    /// At least one vdev points to incorrect location.
    /// If vdev type is File then it means file not found.
    DeviceNotFound,
    /// Trying to create new Zpool, but one or more vdevs already used in
    /// another pool.
    VdevReuse,
    /// Given topology failed validation.
    InvalidTopology,
    /// Failed to parse value. Ideally you never see it, if you see it - it's a
    /// bug.
    ParseError,
    /// Device used in CreateZpoolRequest is smaller than 64M
    DeviceTooSmall,
    /// Permission denied to create zpool. This might happened because:
    /// a) you running it as not root
    /// b) you running it inside jail that isn't allowed to operate zfs
    PermissionDenied,
    /// Trying to pause/stop scrub that either never stared or already completed
    NoActiveScrubs,
    /// Trying to take only device offline.
    NoValidReplicas,
    /// Couldn't parse string to raid type.
    UnknownRaidType,
    /// Cannot attach a device to device that is part of raidz. It can only be
    /// attached to mirrors and top-level disks.
    CannotAttach,
    /// Operation on device that was not found in the pool.
    NoSuchDevice,
    /// Trying to detach a device from vdev without any valid replicas left.
    OnlyDevice,
    /// Trying to add vdev with wring replication level to existing zpool with
    /// different replication level. For example: mirror to zpool.
    MismatchedReplicationLevel,
    /// Cache device must a disk or disk slice.
    InvalidCacheDevice,
    /// Don't know (yet) how to categorize this error. If you see this error -
    /// open an issues.
    Other,
}

impl From<io::Error> for ZpoolError {
    fn from(err: io::Error) -> ZpoolError {
        match err.kind() {
            io::ErrorKind::NotFound => ZpoolError::CmdNotFound,
            _ => ZpoolError::Io(err),
        }
    }
}

impl ZpoolError {
    /// Try to convert stderr into internal error type.
    pub fn from_stderr(stderr_raw: &[u8]) -> ZpoolError {
        let stderr = String::from_utf8_lossy(stderr_raw);
        if RE_REUSE_VDEV.is_match(&stderr) {
            let caps = RE_REUSE_VDEV.captures(&stderr).unwrap();
            ZpoolError::VdevReuse(
                caps.get(1).unwrap().as_str().into(),
                caps.get(2).unwrap().as_str().into(),
            )
        } else if RE_REUSE_VDEV2.is_match(&stderr) {
            let caps = RE_REUSE_VDEV2.captures(&stderr).unwrap();
            ZpoolError::VdevReuse(
                caps.get(1).unwrap().as_str().into(),
                caps.get(2).unwrap().as_str().into(),
            )
        } else if RE_REUSE_VDEV_ZOL.is_match(&stderr) {
            ZpoolError::VdevReuse(String::new(), String::new())
        } else if RE_TOO_SMALL.is_match(&stderr) {
            ZpoolError::DeviceTooSmall
        } else if RE_PERMISSION_DENIED.is_match(&stderr) {
            ZpoolError::PermissionDenied
        } else if RE_NO_ACTIVE_SCRUBS.is_match(&stderr) {
            ZpoolError::NoActiveScrubs
        } else if RE_NO_SUCH_POOL.is_match(&stderr) {
            ZpoolError::PoolNotFound
        } else if RE_NO_VALID_REPLICAS.is_match(&stderr) {
            ZpoolError::NoValidReplicas
        } else if RE_CANNOT_ATTACH.is_match(&stderr) {
            ZpoolError::CannotAttach
        } else if RE_NO_SUCH_DEVICE.is_match(&stderr) {
            ZpoolError::NoSuchDevice
        } else if RE_ONLY_DEVICE.is_match(&stderr) {
            ZpoolError::OnlyDevice
        } else if RE_MISMATCH_REPLICATION.is_match(&stderr) {
            ZpoolError::MismatchedReplicationLevel
        } else if RE_INVALID_CACHE_DEVICE.is_match(&stderr) {
            ZpoolError::InvalidCacheDevice
        } else {
            ZpoolError::Other(stderr.into())
        }
    }
}

/// Type alias to `Result<T, ZpoolError>`.
pub type ZpoolResult<T> = Result<T, ZpoolError>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum OfflineMode {
    /// Device will be taken offline until operator manually bring it back
    /// online.
    Permanent,
    /// Upon reboot, the specified physical device reverts to its previous
    /// state.
    UntilReboot,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum OnlineMode {
    /// Bring device online as is.
    Simple,
    /// Expand the device to use all available space. If the device is part of a
    /// mirror or raidz then all devices must be expanded before the new
    /// space will become available to the pool.
    Expand,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CreateMode {
    /// Forces use of vdevs, even if they appear in use or specify a conflicting
    /// replication level.  Not all devices can be overridden in this manner
    Force,
    Gentle,
}

impl Default for CreateMode {
    fn default() -> CreateMode { CreateMode::Gentle }
}

/// Bring device online as is.
/// Generic interface to manage zpools. End goal is to cover most of `zpool(8)`.
/// Using trait here, so I can mock it in unit tests.
pub trait ZpoolEngine {
    /// Check if pool with given name exists. This should not return
    /// [`ZpoolError::PoolNotFound`](enum.ZpoolError.html) error, instead
    /// it should return `Ok(false)`.
    fn exists<N: AsRef<str>>(&self, name: N) -> ZpoolResult<bool>;
    /// Create new zpool.
    fn create(&self, request: CreateZpoolRequest) -> ZpoolResult<()>;
    /// Version of destroy that doesn't verify if pool exists before removing
    /// it.
    fn destroy_unchecked<N: AsRef<str>>(&self, name: N, force: bool) -> ZpoolResult<()>;
    /// Destroy zpool.
    fn destroy<N: AsRef<str>>(&self, name: N, force: bool) -> ZpoolResult<()> {
        if !self.exists(&name)? {
            return Err(ZpoolError::PoolNotFound);
        }

        self.destroy_unchecked(name, force)
    }
    /// Read properties of the pool.
    fn read_properties_unchecked<N: AsRef<str>>(&self, name: N) -> ZpoolResult<ZpoolProperties>;
    /// Read properties of the pool.
    fn read_properties<N: AsRef<str>>(&self, name: N) -> ZpoolResult<ZpoolProperties> {
        if !self.exists(&name)? {
            return Err(ZpoolError::PoolNotFound);
        }
        self.read_properties_unchecked(name)
    }

    /// Update zpool properties.
    fn update_properties<N: AsRef<str>>(
        &self,
        name: N,
        props: ZpoolPropertiesWrite,
    ) -> ZpoolResult<ZpoolProperties> {
        if !self.exists(&name)? {
            return Err(ZpoolError::PoolNotFound);
        }

        let current = self.read_properties_unchecked(&name)?;

        if current.auto_expand != *props.auto_expand() {
            self.set_unchecked(&name, "autoexpand", props.auto_expand())?;
        }

        if current.auto_replace != *props.auto_replace() {
            self.set_unchecked(&name, "autoreplace", props.auto_replace())?;
        }

        if current.cache_file != *props.cache_file() {
            self.set_unchecked(&name, "cachefile", props.cache_file())?;
        }

        // remove comment
        let desired = if props.comment().is_empty() { None } else { Some(props.comment().clone()) };
        if current.comment != desired {
            self.set_unchecked(&name, "comment", props.comment())?;
        }

        if current.delegation != *props.delegation() {
            self.set_unchecked(&name, "delegation", props.delegation())?;
        }

        if current.fail_mode != *props.fail_mode() {
            self.set_unchecked(&name, "failmode", props.fail_mode())?;
        }

        self.read_properties_unchecked(name)
    }

    /// Internal function used to set values. Should be avoided.
    fn set_unchecked<N: AsRef<str>, P: PropPair>(
        &self,
        name: N,
        key: &str,
        value: &P,
    ) -> ZpoolResult<()>;
    /// Export Pool.
    fn export<N: AsRef<str>>(&self, name: N, force: bool) -> ZpoolResult<()> {
        if !self.exists(&name)? {
            return Err(ZpoolError::PoolNotFound);
        }
        self.export_unchecked(name, force)
    }

    fn export_unchecked<N: AsRef<str>>(&self, name: N, force: bool) -> ZpoolResult<()>;
    /// List of pools available for import in `/dev/` directory.
    fn available(&self) -> ZpoolResult<Vec<Zpool>>;
    /// List of pools available
    fn available_in_dir(&self, dir: PathBuf) -> ZpoolResult<Vec<Zpool>>;

    /// Import pool
    fn import_from_dir<N: AsRef<str>>(&self, name: N, dir: PathBuf) -> ZpoolResult<()>;

    /// Get the detailed health status for the given pools.
    fn status_unchecked<N: AsRef<str>>(&self, name: N) -> ZpoolResult<Zpool>;

    /// Get the detailed health status for the given pool.
    fn status<N: AsRef<str>>(&self, name: N) -> ZpoolResult<Zpool> {
        if !self.exists(&name)? {
            return Err(ZpoolError::PoolNotFound);
        }
        self.status_unchecked(name)
    }
    /// Get a status of each pool active in the system
    fn all(&self) -> ZpoolResult<Vec<Zpool>>;

    ///  Begins a scrub or resumes a paused scrub.  The scrub examines all data
    /// in the specified  pools to verify that it checksums correctly. For
    /// replicated (mirror or raidz) devices, ZFS  automatically repairs any
    /// damage discovered during the scrub.
    fn scrub<N: AsRef<str>>(&self, name: N) -> ZpoolResult<()>;
    ///  Pause scrubbing. Scrub pause state and progress are periodically synced
    /// to disk. If the  system is restarted or pool is exported during a
    /// paused scrub, even after import, scrub  will remain paused until it
    /// is resumed.  Once resumed the scrub will pick up from the
    ///  place where it was last checkpointed to disk.
    fn pause_scrub<N: AsRef<str>>(&self, name: N) -> ZpoolResult<()>;
    ///  Stop scrubbing.
    fn stop_scrub<N: AsRef<str>>(&self, name: N) -> ZpoolResult<()>;
    /// Takes the specified physical device offline. While the device is
    /// offline, no attempt is made to read or write to the device.
    fn take_offline<N: AsRef<str>, D: AsRef<OsStr>>(
        &self,
        name: N,
        device: D,
        mode: OfflineMode,
    ) -> ZpoolResult<()>;
    /// Brings the specified physical device online.
    fn bring_online<N: AsRef<str>, D: AsRef<OsStr>>(
        &self,
        name: N,
        device: D,
        mode: OnlineMode,
    ) -> ZpoolResult<()>;

    /// Attaches new_device (disk) to an existing zpool device (VDEV). The
    /// existing device cannot be part of a raidz configuration. If device
    /// is not currently part of a mirrored configuration,
    /// device automatically transforms into a two-way mirror of device and
    /// new_device.
    fn attach<N: AsRef<str>, D: AsRef<OsStr>>(
        &self,
        name: N,
        device: D,
        new_device: D,
    ) -> ZpoolResult<()>;
    ///Detaches device from a mirror. The operation is refused if there are no
    /// other valid replicas of the data.
    ///
    /// * `name` - Name of the zpool
    /// * `device` - Name of the device or path to sparse file.
    fn detach<N: AsRef<str>, D: AsRef<OsStr>>(&self, name: N, device: D) -> ZpoolResult<()>;

    /// Add a VDEV to existing Zpool.
    ///
    /// * `name` - Name of the zpool
    /// * `new_vdev` - New VDEV
    /// * `add_mode` - Disable some safety checks
    fn add_vdev<N: AsRef<str>>(
        &self,
        name: N,
        new_vdev: CreateVdevRequest,
        add_mode: CreateMode,
    ) -> ZpoolResult<()>;

    /// Add a ZIL to existing Zpool.
    ///
    /// * `name` - Name of the zpool
    /// * `new_zil` - VDEV to use as ZIL
    /// * `add_mode` - Disable some safety checks
    fn add_zil<N: AsRef<str>>(
        &self,
        name: N,
        new_zil: CreateVdevRequest,
        add_mode: CreateMode,
    ) -> ZpoolResult<()>;

    /// Add a cache to existing Zpool.
    ///
    /// * `name` - Name of the zpool
    /// * `new_cache` - Disk to use as cache
    /// * `add_mode` - Disable some safety checks
    fn add_cache<N: AsRef<str>, D: AsRef<OsStr>>(
        &self,
        name: N,
        new_cache: D,
        add_mode: CreateMode,
    ) -> ZpoolResult<()>;

    /// Remove Spare, Cache or log device
    ///
    /// * `name` - Name of the zpool
    /// * `device` - Name of the device or path to sparse file.
    fn remove<N: AsRef<str>, D: AsRef<OsStr>>(&self, name: N, device: D) -> ZpoolResult<()>;
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

        let vdev_reuse_text = b"cannot create \'tests-8804202574521870666\': one or more vdevs refer to the same device, or one of\nthe devices is part of an active md or lvm device\n";
        let err = ZpoolError::from_stderr(vdev_reuse_text);
        assert_eq!(ZpoolErrorKind::VdevReuse, err.kind());

        let vdev_reuse_text = b"invalid vdev specification\nuse '-f' to override the following errors:\n/vdevs/vdev0 is part of potentially active pool 'tests-9706865472708603696'\n";
        let err = ZpoolError::from_stderr(vdev_reuse_text);
        assert_eq!(ZpoolErrorKind::VdevReuse, err.kind());

        // TODO(#49): add regexp for this too
        //let vdev_reuse_text = b"invalid vdev specification\nuse \'-f\' to override
        // the following errors:\n/vdevs/vdev0 is part of exported pool \'test\'\n";
        // let err = ZpoolError::from_stderr(vdev_reuse_text);
        //assert_eq!(ZpoolErrorKind::VdevReuse, err.kind());
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

    //noinspection RsTypeCheck
    #[test]
    fn num_error_from() {
        let int_err = "as".parse::<i8>().unwrap_err();
        let float_err = "as".parse::<f32>().unwrap_err();

        let err = ZpoolError::from(int_err);
        assert_eq!(ZpoolErrorKind::ParseError, err.kind());

        let err = ZpoolError::from(float_err);
        assert_eq!(ZpoolErrorKind::ParseError, err.kind());
    }

    #[test]
    fn too_small() {
        let text = b"cannot create \'tests-5825559772339520034\': one or more devices is less than the minimum size (64M)\n";
        let err = ZpoolError::from_stderr(text);

        assert_eq!(ZpoolErrorKind::DeviceTooSmall, err.kind());
    }

    #[test]
    fn permission_denied() {
        let text = b"cannot create \'tests-10742509212158788460\': permission denied\n";
        let err = ZpoolError::from_stderr(text);

        assert_eq!(ZpoolErrorKind::PermissionDenied, err.kind());
    }

    #[test]
    fn no_active_scrubs() {
        let text = b"cannot pause scrubbing hell: there is no active scrub\n";
        let err = ZpoolError::from_stderr(text);
        assert_eq!(ZpoolErrorKind::NoActiveScrubs, err.kind());

        let text = b"cannot cancel scrubbing hell: there is no active scrub\n";
        let err = ZpoolError::from_stderr(text);
        assert_eq!(ZpoolErrorKind::NoActiveScrubs, err.kind());
    }

    #[test]
    fn no_such_pool() {
        let text = b"cannot open 'hellasd': no such pool\n";
        let err = ZpoolError::from_stderr(text);
        assert_eq!(ZpoolErrorKind::PoolNotFound, err.kind());
    }

    #[test]
    fn no_valid_replicas() {
        let text = b"cannot offline /vdevs/vdev0: no valid replicas\n";
        let err = ZpoolError::from_stderr(text);
        assert_eq!(ZpoolErrorKind::NoValidReplicas, err.kind());
    }

    #[test]
    fn test_unknown_raid_type() {
        use std::str::FromStr;
        let kind = crate::zpool::VdevType::from_str("notzraid");
        assert_eq!(ZpoolErrorKind::UnknownRaidType, kind.unwrap_err().kind());
    }

    #[test]
    fn test_cannot_attach() {
        let text = b"cannot attach /vdevs/vdev3 to /vdevs/vdev0: can only attach to mirrors and top-level disks";
        let err = ZpoolError::from_stderr(text);
        assert_eq!(ZpoolErrorKind::CannotAttach, err.kind());
    }

    #[test]
    fn test_no_such_device() {
        let text = b"cannot attach /vdevs/vdev3 to /vdevs/vdev6: no such device in pool";
        let err = ZpoolError::from_stderr(text);
        assert_eq!(ZpoolErrorKind::NoSuchDevice, err.kind());
    }

    #[test]
    fn test_only_device() {
        let text = b"cannot detach /vdevs/vdev0: only applicable to mirror and replacing vdevs";
        let err = ZpoolError::from_stderr(text);
        assert_eq!(ZpoolErrorKind::OnlyDevice, err.kind());
    }

    #[test]
    fn test_mismatched_replication() {
        let text = b"invalid vdev specification\nuse \'-f\' to override the following errors:\nmismatched replication level: pool uses raidz and new vdev is mirror";
        let err = ZpoolError::from_stderr(text);
        assert_eq!(ZpoolErrorKind::MismatchedReplicationLevel, err.kind());
    }

    #[test]
    fn test_invalid_cache_device() {
        let text = b"cannot add to 'asd': cache device must be a disk or disk slice\n?";
        let err = ZpoolError::from_stderr(text);
        assert_eq!(ZpoolErrorKind::InvalidCacheDevice, err.kind());
    }
}
