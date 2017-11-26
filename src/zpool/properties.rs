/// Property related stuff.

use std::path::PathBuf;
/// Represent state of zpool or vdev. Read
/// [more](https://docs.oracle.com/cd/E19253-01/819-5461/gamno/index.html).
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Health {
    /// Healthy and operational
    Online,
    /// Unhealthy, but ooperational,
    Degraded,
    /// Not operational
    Faulted,
    /// Taken offline by admin
    Offline,
    /// Can't open device.
    Unavaible,
    /// Phusically removed while the sytem was running.
    Removed,
}

/// Controls the system behavior in the event of catastrophic pool failure.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FailMode {
    /// Blocks all I/O access until the device connectivity is recovered and
    /// the errors are
    /// cleared.  This is the default behavior.
    Wait,
    /// Returns EIO to any new write I/O requests but allows reads to any of
    /// the remaining healthy
    /// devices. Any write requests that have yet to be committed to disk would
    /// be blocked.
    Continue,
    /// Prints out a message to the console and generates a system
    /// crash dump.
    Panic,
}

/// Where to store cache for zpool.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CacheType {
    /// Default location.
    Default,
    /// No cache.
    None,
    /// Custom location.
    Custom(String),
}

/// Available properties for write. See `zpool(8)` for more information.
///
/// ```rust
/// use libzfs::zpool::ZpoolPropertiesWriteBuilder;
/// use std::path::PathBuf;
/// use libzfs::zpool::CacheType;
///
/// let props = ZpoolPropertiesWriteBuilder::default()
///                 .alt_root(PathBuf::from("/mnt/ext_pool"))
///                 .build()
///                 .unwrap();
///
/// assert!(!props.read_only());
/// assert!(props.boot_fs().is_none());
/// assert_eq!(props.cache_file(), &CacheType::Default);
///
/// let props = ZpoolPropertiesWriteBuilder::default().build();
/// assert!(props.is_ok());
/// ```
#[derive(Getters, Builder, Debug, Clone, PartialEq)]
pub struct ZpoolPropertiesWrite {
    /// Alternate root directory, can only be set during creation or import.
    /// Ignored in other
    /// cases.
    #[builder(default)]
    #[builder(setter(into))]
    alt_root: Option<PathBuf>,

    /// Pool is read only
    #[builder(default="false")]
    read_only: bool,

    /// Controls automatic pool expansion when the underlying LUN is grown.
    #[builder(default="false")]
    auto_expand: bool,

    /// Controls automatic device replacement. If set to "on", any new device,
    /// found in the
    /// same physical location as a device that previously belonged to the
    /// pool, is automatically
    /// formatted and replaced. The default behavior is "off".
    #[builder(default="false")]
    auto_replace: bool,

    ///  Identifies the default bootable dataset for the root pool.
    #[builder(default)]
    boot_fs: Option<String>,

    /// Controls the location of where the pool configuration is cached.
    #[builder(default="CacheType::Default")]
    cache_file: CacheType,
    /// Threshold for the number of block ditto copies. If the reference
    /// count for a deduplicated block increases above this number, a new
    /// ditto copy of this block is automatically stored. Default setting is
    /// 0 which causes no ditto copies to be created for deduplicated blocks.
    /// The miniumum legal nonzero setting is 100.
    #[builder(default = "0")]
    dedup_ditto: u64,
    /// Controls whether a non-privileged user is granted access based on the
    /// dataset permissions defined on the dataset. See zfs(8) for more
    /// information on ZFS delegated administration.
    #[builder(default="false")]
    delegation: bool,
    /// Controls the system behavior in the event of catastrophic pool
    /// failure. This condition is typically a result of a loss of
    /// connectivity to the underlying storage device(s) or a failure of all
    /// devices within the pool.
    #[builder(default = "FailMode::Wait")]
    fail_mode: FailMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZpoolProperties {
    /// Amount of storage space within the pool that has been physically
    /// allocated.
    pub alloc: usize,
    /// Percentage of pool space used. Percentage.
    pub capacity: u8,
    /// A text string consisting of printable ASCII characters that will be
    /// stored such that it is
    /// available even if the pool becomes faulted.  An administrator can
    /// provide additional information about a pool using this property.
    pub comment: String,
    /// The deduplication ratio specified for a pool, expressed as a
    /// multiplier.  For example,
    /// a dedupratio value of 1.76 indicates that 1.76 units of data were
    /// stored but only 1 unit
    /// of disk space was actually consumed. See `zfs(8)` for a description of
    /// the deduplication feature.
    pub dedup_ratio: f64,
    /// Amount of uninitialized space within the pool or device that
    /// can be used to increase the total capacity of the pool.
    /// Uninitialized space consists of any space on an EFI labeled
    /// vdev which has not been brought online (i.e. zpool online
    /// -e).  This space occurs when a LUN is dynamically expanded.
    pub expand_size: usize,
    /// The amount of fragmentation in the pool. In percents.
    pub fragmentation: i8,
    /// Number of blocks within the pool that are not allocated.
    pub free: i64,
    ///  After a file system or snapshot is destroyed, the space it
    ///  was using is returned to the pool asynchronously.  freeing is
    /// the amount of space remaining to be reclaimed.  Over time
    /// freeing will decrease while free increases.
    pub freeing: i64,
    /// A unique identifier for the pool.
    pub guid: String,
    /// The current health of the pool.
    pub health: Health,
    /// Total size of the storage pool.
    pub size: usize,
    // writable
    /// Alternate root directory, can only be set during creation or import.
    pub alt_root: PathBuf,
    /// Pool is read only
    pub read_only: bool,
    /// Controls automatic pool expansion when the underlying LUN is grown.
    pub auto_expand: bool,
    /// Controls automatic device replacement. If set to "on", any new device,
    /// found in the
    /// same physical location as a device that previously belonged to the
    /// pool, is automatically
    /// formatted and replaced. The default behavior is "off".
    pub auto_replace: bool,
    ///  Identifies the default bootable dataset for the root pool.
    pub boot_fs: String,
    /// Controls the location of where the pool configuration is cached.
    pub cache_file: CacheType,
    /// Threshold for the number of block ditto copies. If the reference
    /// count for a deduplicated block increases above this number, a new
    /// ditto copy of this block is automatically stored. Default setting is
    /// 0 which causes no ditto copies to be created for deduplicated blocks.
    /// The miniumum legal nonzero setting is 100.
    pub dedup_ditto: usize,
    /// Controls whether a non-privileged user is granted access based on the
    /// dataset permissions defined on the dataset. See `zfs(8)` for more
    /// information on ZFS delegated administration.
    pub delegation: bool,
    /// Controls the system behavior in the event of catastrophic pool
    /// failure. This condition is typically a result of a loss of
    /// connectivity to the underlying storage device(s) or a failure of all
    /// devices within the pool.
    pub fail_mode: FailMode,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_defaults() {
        let built = ZpoolPropertiesWriteBuilder::default().build().unwrap();
        let handmade = ZpoolPropertiesWrite {
            alt_root: None,
            read_only: false,
            auto_expand: false,
            auto_replace: false,
            boot_fs: None,
            cache_file: CacheType::Default,
            dedup_ditto: 0,
            delegation: false,
            fail_mode: FailMode::Wait,
        };

        assert_eq!(handmade, built);
    }
}
