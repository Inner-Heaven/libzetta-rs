use std::{default::Default, path::PathBuf};
use strum_macros::{AsRefStr, Display, EnumString};

use crate::zfs::description::DatasetKind;

macro_rules! impl_zfs_prop {
    ($type_:ty, $as_str:literal) => {
        impl ZfsProp for $type_ {
            fn as_nv_key() -> &'static str {
                $as_str
            }
            fn as_nv_value(&self) -> u64 {
                *self as u64
            }
        }
    }
}
pub trait ZfsProp {
    /// String representation of ZFS Property
    fn as_nv_key() -> &'static str;
    fn as_nv_value(&self) -> u64;
}
/// Controls how ACL entries inherited when files and directories created. Default value is
/// `Restricted`.
#[derive(AsRefStr, EnumString, Display, Eq, PartialEq, Debug, Clone, Copy)]
#[repr(u64)]
pub enum AclInheritMode {
    /// For new objects, no ACL entries inherited when a file or directory created. The ACL on the
    /// new file or directory is equal to the permissions of the file or directory.
    #[strum(serialize = "discard")]
    Discard = 0,

    /// For new objects, only inheritable ACL entries that have an access type of `deny` are
    /// inherited.
    #[strum(serialize = "noallow")]
    Noallow = 1,

    /// For new objects, the `write_owner` and `write_acl` permissions removed when an ACL entry
    /// inherited.
    #[strum(serialize = "restricted")]
    Restricted = 4,

    /// (Deprecated, might not work on all platforms) For new objects, the `write_owner` and `write_acl` permissions removed when an ACL entry
    /// inherited.
    #[strum(serialize = "secure")]
    Secure = 2,

    /// When the property value set to passthrough, files created with permissions determined by
    /// the inheritable ACEs. If no inheritable ACEs exist that affect the permissions, then the
    /// permissions set in accordance to the requested permissions from the application.
    #[strum(serialize = "passthrough")]
    Passthrough = 3,

    #[strum(serialize = "passthrough-x")]
    PassthroughX = 5,
}

impl Default for AclInheritMode {
    fn default() -> AclInheritMode { AclInheritMode::Restricted }
}

/// This property modifies ACL behavior when a file initially created or whenever a file or
/// directory's permissions modified by the chmod command.
#[derive(AsRefStr, EnumString, Display, Eq, PartialEq, Debug, Clone, Copy)]
#[repr(u64)]
pub enum AclMode {
    /// All ACL entries removed except for the entries needed to define the mode of the file or
    /// directory.
    #[strum(serialize = "discard")]
    Discard = 0,
    /// User or group ACL permissions reduced so that they are no greater than the group
    /// permissions, unless it is a user entry that has the same UID as the owner of the file or
    /// directory. Then, the ACL permissions reduced so that they are no greater than the owner
    /// permissions.
    #[strum(serialize = "groupmask")]
    GroupMask = 2,

    /// During a `chmod` operation, ACEs other than `owner@`, `group@`, or `everyone@` are not
    /// modified in any way. ACEs with `owner@`, `group@`, or `everyone@` are disabled to set the
    /// file mode as requested by the `chmod` operation.
    #[strum(serialize = "passthrough")]
    Passthrough = 3,

    #[strum(serialize = "restricted")]
    Restricted = 4,
}

impl Default for AclMode {
    fn default() -> AclMode { AclMode::Discard }
}

/// Controls the checksum used to verify data integrity. Default value is `on`.
///
/// NOTE: Some variants might not be supported by underlying zfs module. Consult proper manual pages
/// before using anything other than `on`.
#[derive(AsRefStr, EnumString, Display, Eq, PartialEq, Debug, Clone, Copy)]
#[repr(u64)]
pub enum Checksum {
    /// Use value from the parent
    #[strum(serialize = "inherit")]
    Inherit = 0,
    /// Auto-select most appropriate algorithm. Currently, it is `fletcher4`.
    #[strum(serialize = "on")]
    On = 1,
    /// Disable integrity check. Not recommended at all.
    #[strum(serialize = "off")]
    Off = 2,
    /// Not only disables integrity but also disables maintaining parity for user data. This
    /// setting used internally by a dump device residing on a RAID-Z pool and should not be used
    /// by any other dataset.
    #[strum(serialize = "noparity")]
    NoParity = 10,
    #[strum(serialize = "fletcher2")]
    Fletcher2 = 6 ,
    #[strum(serialize = "fletcher4")]
    Fletcher4 = 7,
    #[strum(serialize = "sha256")]
    SHA256 = 8,
    #[strum(serialize = "sha512")]
    SHA512 = 11,
    #[strum(serialize = "skein")]
    Skein = 12,
}


impl Default for Checksum {
    fn default() -> Self { Checksum::On }
}

/// Enables or disables compression for a dataset.
///
/// NOTE: Some variants might not be supported by underlying zfs module. Consult proper manual pages
/// before using anything other than `off`.
#[derive(AsRefStr, EnumString, Display, Eq, PartialEq, Debug, Clone, Copy)]
#[repr(u64)]
pub enum Compression {
    /// Use value from the parent
    #[strum(serialize = "inherit")]
    Inherit = 0,
    /// Auto-select most appropriate algorithm. If possible uses LZ4, if not then LZJB.
    #[strum(serialize = "on")]
    On = 1,
    /// Disables compression.
    #[strum(serialize = "off")]
    Off = 2,
    #[strum(serialize = "lzjb")]
    LZJB = 3,
    /// The lz4 compression algorithm is a high-performance replacement for the lzjb algorithm.
    #[strum(serialize = "lz4")]
    LZ4 = 15,
    /// The zle compression algorithm compresses runs of zeros.
    #[strum(serialize = "lze")]
    LZE = 14,
    /// Fastest gzip level
    #[strum(serialize = "gzip-1")]
    Gzip1 = 5,
    #[strum(serialize = "gzip-2")]
    Gzip2 = 6,
    #[strum(serialize = "gzip-3")]
    Gzip3 = 7,
    #[strum(serialize = "gzip-4")]
    Gzip4 = 8,
    #[strum(serialize = "gzip-5")]
    Gzip5 = 9,
    #[strum(serialize = "gzip-6")]
    Gzip6 = 10,
    #[strum(serialize = "gzip-7")]
    Gzip7 = 11,
    #[strum(serialize = "gzip-8")]
    Gzip8 = 12,
    /// Slowest gzip level
    #[strum(serialize = "gzip-9")]
    Gzip9 = 13,
}

impl Default for Compression {
    fn default() -> Self { Compression::Off }
}
/// Sets the number of copies of user data per file system. These copies are in addition to any
/// pool-level redundancy.
#[derive(AsRefStr, EnumString, Display, Eq, PartialEq, Debug, Clone, Copy)]
#[repr(u64)]
pub enum Copies {
    #[strum(serialize = "1")]
    One = 1,
    #[strum(serialize = "2")]
    Two,
    #[strum(serialize = "3")]
    Three,
}

impl Default for Copies {
    fn default() -> Self { Copies::One }
}

/// What is cached in the primary cache (ARC).
#[derive(AsRefStr, EnumString, Display, Eq, PartialEq, Debug, Clone, Copy)]
#[repr(u64)]
pub enum CacheMode {
    /// Both user data and metadata.
    #[strum(serialize = "all")]
    All = 2,
    /// Just the metadata.
    #[strum(serialize = "metadata")]
    Metadata = 1,
    /// Neither user data nor metadata is cached.
    #[strum(serialize = "none")]
    None = 0,
}

impl ZfsProp for CacheMode {
    fn as_nv_key() -> &'static str {
        unimplemented!()
    }

    fn as_nv_value(&self) -> u64 {
        *self as u64
    }
}

impl Default for CacheMode {
    fn default() -> Self { CacheMode::All }
}

/// Controls whether the .zfs directory is hidden or visible in the root of the file system
#[derive(AsRefStr, EnumString, Display, Eq, PartialEq, Debug, Clone, Copy)]
#[repr(u64)]
pub enum SnapDir {
    #[strum(serialize = "hidden")]
    Hidden = 0,
    #[strum(serialize = "visible")]
    Visible = 1,
}

impl Default for SnapDir {
    fn default() -> Self { SnapDir::Hidden }
}


#[derive(AsRefStr, EnumString, Display, Eq, PartialEq, Debug, Clone, Copy)]
#[repr(u64)]
pub enum CanMount {
    /// Allowed to be mounted
    #[strum(serialize = "on")]
    On = 1,
    /// Can't be mounted
    #[strum(serialize = "off")]
    Off = 0,
    /// Can be mounted, but only explicitly
    #[strum(serialize = "noauto")]
    NoAuto = 2,
}

impl Default for CanMount {
    fn default() -> Self {
        CanMount::On
    }
}

/// Most of native properties of dataset - both immutable and mutable. Default values taken from FreeBSD 12.
///
/// Notable missing properties:
///  - shareiscsi
///  - sharenfs
///  - sharesmb
///  - version
///  - zoned
#[derive(Debug, Clone, PartialEq, Getters)]
#[get = "pub"]
pub struct DatasetProperties {
    /// Controls how ACL entries inherited when files and directories created.
    acl_inherit: AclInheritMode,
    /// Controls how an ACL entry modified during a `chmod` operation.
    acl_mode: AclMode,
    /// Controls whether the access time for files updated when they are read.
    atime: bool,
    /// Read-only property that identifies the amount of disk space available to a dataset and all
    /// its children, assuming no other activity in the pool. Because disk space shared within a
    /// pool, available space can be limited by various factors including physical pool size,
    /// quotas, reservations, and other datasets within the pool.
    available: i64,
    /// Controls whether a file system can be mounted.
    can_mount: CanMount,
    /// Controls the checksum used to verify data integrity.
    checksum: Checksum,
    /// Enables or disables compression for a dataset.
    compression: Compression,
    /// Read-only property that identifies the compression ratio achieved for a dataset, expressed
    /// as a multiplier.
    compression_ratio: f64,
    /// Sets the number of copies of user data per file system. Available values are 1, 2, or 3.
    /// These copies are in addition to any pool-level redundancy. Disk space used by multiple
    /// copies of user data charged to the corresponding file and dataset, and counts against
    /// quotas and reservations. In addition, the used property updated when multiple copies
    /// enabled. Consider setting this property when the file system created because changing this
    /// property on an existing file system only affects newly written data.
    copies: Copies,
    /// Read-only property that identifies the date and time a dataset created.
    creation: String,
    /// Controls whether device files in a file system can be opened.
    devices: bool,
    /// Controls whether programs in a file system allowed to be executed. Also, when set to
    /// `false`, `mmap(2)` calls with `PROT_EXEC` disallowed.
    exec: bool,
    /// Read-only property that indicates whether a file system, clone, or snapshot is currently
    /// mounted. This property does not apply to volumes.
    mounted: bool,
    /// Controls the mount point used for this file system.
    mount_point: Option<PathBuf>,
    /// Controls what is cached in the primary cache (ARC).
    primary_cache: CacheMode,
    // Read-only property for cloned file systems or volumes that identifies the snapshot from
    // which the clone was created.
    origin: Option<String>,
    /// Limits the amount of disk space a dataset and its descendants can consume.
    quota: Option<u64>,
    /// Controls whether a dataset can be modified.
    readonly: bool,
    /// Specifies a suggested block size for files in a file system in bytes. The size specified
    /// must be a power of two greater than or equal to 512 and less than or equal to 128 KiB.
    /// If the large_blocks feature is enabled on the pool, the size may be up to 1 MiB.
    record_size: u64,
    /// Read-only property that identifies the amount of data accessible by a dataset, which might
    /// or might not be shared with other datasets in the pool.
    referenced: u64,
    /// Sets the amount of disk space a dataset can consume. This property enforces a hard limit on
    /// the amount of space used. This hard limit does not include disk space used by descendents,
    /// such as snapshots and clones.
    ref_quota: Option<u64>,
    /// Sets the minimum amount of disk space is guaranteed to a dataset, not including
    /// descendants, such as snapshots and clones.
    ref_reservation: Option<u64>,
    /// Sets the minimum amount of disk space guaranteed to a dataset and its descendants.
    reservation: Option<u64>,
    /// Controls what is cached in the secondary cache (L2ARC).
    secondary_cache: CacheMode,
    /// Controls whether the `setuid` bit is honored in a file system.
    setuid: bool,
    /// Controls whether the .zfs directory is hidden or visible in the root of the file system
    snap_dir: SnapDir,
    /// Read-only property that identifies the dataset type as filesystem (file system or clone),
    /// volume, or snapshot.
    kind: DatasetKind,
    /// Read-only property that identifies the amount of disk space consumed by a dataset and all
    /// its descendants.
    used: u64,
    /// Read-only property that identifies the amount of disk space is used by children of this
    /// dataset, which would be freed if all the dataset's children were destroyed.
    used_by_children: u64,
    /// Read-only property that identifies the amount of disk space is used by a dataset itself.
    used_by_dataset: u64,
    /// Read-only property that identifies the amount of disk space is used by a refreservation set
    /// on a dataset.
    used_by_ref_reservation: u64,
    /// Read-only property that identifies the amount of disk space is consumed by snapshots of a
    /// dataset.
    used_by_snapshots: u64,
    /// For volumes, specifies the logical size of the volume.
    volume_size: u64,
    /// For volumes, specifies the block size of the volume in bytes. The block size cannot be
    /// changed after the volume has been written, so set the block size at volume creation time.
    /// The default block size for volumes is 8 KB. Any power of 2 from 512 bytes to 128 KB is
    /// valid.
    volume_block_size: u64,
    /// Indicates whether extended attributes are enabled or disabled.
    xattr: bool,
}

impl_zfs_prop!(AclInheritMode, "aclinherit");
impl_zfs_prop!(AclMode, "aclmode");
impl_zfs_prop!(CanMount, "canmount");
impl_zfs_prop!(Checksum, "checksum");
impl_zfs_prop!(Compression, "compression");
impl_zfs_prop!(Copies, "copies");
impl_zfs_prop!(SnapDir, "snapdir");
