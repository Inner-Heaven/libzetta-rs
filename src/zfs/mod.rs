use std::path::PathBuf;
use std::io;

pub mod description;
pub use description::{Dataset, DatasetKind};

pub mod delegating;
pub use delegating::DelegatingZfsEngine;
pub mod open3;
pub use open3::ZfsOpen3;

pub mod lzc;
pub use lzc::ZfsLzc;
use std::collections::{HashMap};
use crate::zfs::properties::{AclInheritMode, AclMode};

pub mod properties;
pub use properties::{CanMount, SnapDir, DatasetProperties, Checksum, Copies, Compression, CacheMode};

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        /// `zfs not found in the PATH. Open3 specific error.
        CmdNotFound {}
        LZCInitializationFailed(err: std::io::Error) {
            cause(err)
        }
        NvOpError(err: libnv::NvError) {
            cause(err)
            from()
        }
        InvalidInput {}
        Io(err: std::io::Error) {
            cause(err)
        }
        Unknown {}
    }
}

impl From<io::Error> for Error {
    #[allow(clippy::wildcard_enum_match_arm)]
    fn from(err: io::Error) -> Error {
        match err.kind() {
            io::ErrorKind::NotFound => Error::CmdNotFound,
            io::ErrorKind::InvalidInput => Error::InvalidInput,
            _ => Error::Io(err),
        }
    }
}

type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    pub fn kind(&self) -> ErrorKind {
        match self {
            Error::CmdNotFound => ErrorKind::CmdNotFound,
            Error::LZCInitializationFailed(_) => ErrorKind::LZCInitializationFailed,
            Error::NvOpError(_) => ErrorKind::NvOpError,
            Error::InvalidInput => ErrorKind::InvalidInput,
            Error::Io(_) => ErrorKind::Io,
            Error::Unknown => ErrorKind::Unknown,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ErrorKind {
    CmdNotFound,
    LZCInitializationFailed,
    NvOpError,
    InvalidInput,
    Io,
    Unknown,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
    }
}

pub trait ZfsEngine {
    /// Check if a dataset (a filesystem, or a volume, or a snapshot with the given name exists.
    ///
    /// NOTE: Can't be used to check for existence of bookmarks.
    ///  * `name` - The dataset name to check.
    fn exists<N: Into<PathBuf>>(&self, name: N) -> Result<bool>;

    /// Create a new dataset.
    fn create(&self, request: CreateDatasetRequest) -> Result<()>;

    /// Deletes the dataset
    fn destroy<N: Into<PathBuf>>(&self, name: N) -> Result<()>;

    //fn list(pool: Option<String>) -> Result<Vec<Dataset>>;
}

#[derive(Default, Builder, Debug, Clone, Getters)]
#[builder(setter(into))]
#[get = "pub"]
/// Consumer friendly builder for NvPair. Use this to create your datasets. Some properties only work on filesystems, some only on volumes.
pub struct CreateDatasetRequest {
    /// Name of the dataset. First crumb of path is name of zpool.
    name: PathBuf,
    /// Filesystem or Volume.
    kind: DatasetKind,
    /// Optional user defined properties. User property names must conform to the following characteristics:
    ///
    ///  - Contain a colon (':') character to distinguish them from native properties.
    ///  - Contain lowercase letters, numbers, and the following punctuation characters: ':', '+','.', '_'.
    ///  - Maximum user property name is 256 characters.
    #[builder(default)]
    user_properties: Option<HashMap<String, String>>,

    //
    // the rest is zfs native properties
    //

    /// Controls how ACL entries inherited when files and directories created.
    #[builder(default)]
    acl_inherit: AclInheritMode,
    /// Controls how an ACL entry modified during a `chmod` operation.
    #[builder(default)]
    acl_mode: AclMode,
    /// Controls whether the access time for files updated when they are read.
    #[builder(default = "true")]
    atime: bool,
    /// Controls whether a file system can be mounted.
    #[builder(default)]
    can_mount: CanMount,
    /// Controls the checksum used to verify data integrity.
    #[builder(default)]
    checksum: Checksum,
    /// Enables or disables compression for a dataset.
    #[builder(default)]
    compression: Compression,
    /// Sets the number of copies of user data per file system. Available values are 1, 2, or 3.
    /// These copies are in addition to any pool-level redundancy. Disk space used by multiple
    /// copies of user data charged to the corresponding file and dataset, and counts against
    /// quotas and reservations. In addition, the used property updated when multiple copies
    /// enabled. Consider setting this property when the file system created because changing this
    /// property on an existing file system only affects newly written data.
    #[builder(default)]
    copies: Copies,
    /// Controls whether device files in a file system can be opened.
    #[builder(default = "true")]
    devices: bool,
    /// Controls whether programs in a file system allowed to be executed. Also, when set to
    /// `false`, `mmap(2)` calls with `PROT_EXEC` disallowed.
    #[builder(default = "true")]
    exec: bool,
    /// Controls the mount point used for this file system.
    #[builder(default)]
    mount_point: Option<PathBuf>,
    /// Controls what is cached in the primary cache (ARC).
    #[builder(default)]
    primary_cache: CacheMode,
    /// Limits the amount of disk space a dataset and its descendants can consume.
    #[builder(default)]
    quota: Option<u64>,
    /// Controls whether a dataset can be modified.
    #[builder(default = "false")]
    readonly: bool,
    /// Specifies a suggested block size for files in a file system in bytes. The size specified
    /// must be a power of two greater than or equal to 512 and less than or equal to 128 KiB.
    /// If the large_blocks feature is enabled on the pool, the size may be up to 1 MiB.
    #[builder(default)]
    record_size: Option<u64>,
    /// Sets the amount of disk space a dataset can consume. This property enforces a hard limit on
    /// the amount of space used. This hard limit does not include disk space used by descendents,
    /// such as snapshots and clones.
    #[builder(default)]
    ref_quota: Option<u64>,
    /// Sets the minimum amount of disk space is guaranteed to a dataset, not including
    /// descendants, such as snapshots and clones.
    #[builder(default)]
    ref_reservation: Option<u64>,
    /// Sets the minimum amount of disk space guaranteed to a dataset and its descendants.
    #[builder(default)]
    reservation: Option<u64>,
    /// Controls what is cached in the secondary cache (L2ARC).
    #[builder(default)]
    secondary_cache: CacheMode,
    /// Controls whether the `setuid` bit is honored in a file system.
    #[builder(default = "true")]
    setuid: bool,
    /// Controls whether the .zfs directory is hidden or visible in the root of the file system
    #[builder(default)]
    snap_dir: SnapDir,
    /// For volumes, specifies the logical size of the volume.
    #[builder(default)]
    volume_size: Option<u64>,
    /// For volumes, specifies the block size of the volume in bytes. The block size cannot be
    /// changed after the volume has been written, so set the block size at volume creation time.
    /// The default block size for volumes is 8 KB. Any power of 2 from 512 bytes to 128 KB is
    /// valid.
    #[builder(default)]
    volume_block_size: Option<u64>,
    /// Indicates whether extended attributes are enabled or disabled.
    #[builder(default = "true")]
    xattr: bool,
}

impl CreateDatasetRequest {
    pub fn builder() -> CreateDatasetRequestBuilder {
        CreateDatasetRequestBuilder::default()
    }
}