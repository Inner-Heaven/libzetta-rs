/// Property related stuff.

use super::{ZpoolResult, ZpoolError};
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
    Unavailable,
    /// Phusically removed while the sytem was running.
    Removed,
}

impl Health {
    /// parse str to Health.
    pub fn try_from_str(val: Option<&str>) -> ZpoolResult<Health> {
        let val_str = val.ok_or(ZpoolError::ParseError)?;
        match val_str {
            "ONLINE" => Ok(Health::Online),
            "DEGRADED" => Ok(Health::Degraded),
            "FAULTED" => Ok(Health::Faulted),
            "OFFLINE"=> Ok(Health::Offline),
            "UNAVAIL" => Ok(Health::Unavailable),
            "REMOVED" => Ok(Health::Removed),
            _   => Err(ZpoolError::ParseError)
        }
    }
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
impl FailMode {
    /// parse str to FailMode.
    pub fn try_from_str(val: Option<&str>) -> ZpoolResult<FailMode> {
        let val_str = val.ok_or(ZpoolError::ParseError)?;
        match val_str {
            "wait" => Ok(FailMode::Wait),
            "continue" => Ok(FailMode::Continue),
            "panic" => Ok(FailMode::Panic),
            _   => Err(ZpoolError::ParseError),
        }
    }
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

impl CacheType {
    /// parse str to CacheType.
    pub fn try_from_str(val: Option<&str>) -> ZpoolResult<CacheType> {
        let val_str = val.ok_or(ZpoolError::ParseError)?;
        match val_str {
            "-" | "" => Ok(CacheType::Default),
            "none" => Ok(CacheType::None),
            n   => Ok(CacheType::Custom(String::from(n))),
        }
    }
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
    pub comment: Option<String>,
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
    pub expand_size: Option<usize>,
    /// The amount of fragmentation in the pool. In percents.
    pub fragmentation: i8,
    /// Number of blocks within the pool that are not allocated.
    pub free: i64,
    ///  After a file system or snapshot is destroyed, the space it
    ///  was using is returned to the pool asynchronously.  freeing is
    /// the amount of space remaining to be reclaimed.  Over time
    /// freeing will decrease while free increases.
    pub freeing: i64,
    /// A unique identifier for the pool. Technically this is i128...
    pub guid: String,
    /// The current health of the pool.
    pub health: Health,
    /// Total size of the storage pool.
    pub size: usize,
    /// Leaked space?
    pub leaked: usize,
    // writable
    /// Alternate root directory, can only be set during creation or import.
    pub alt_root: Option<PathBuf>,
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
    pub boot_fs: Option<String>,
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

fn parse_bool(val: Option<&str>) -> ZpoolResult<bool> {
    let val_str = val.ok_or(ZpoolError::ParseError)?;
    match val_str {
        "off" => Ok(false),
        "on" => Ok(true),
        _ => Err(ZpoolError::ParseError),
    }
}
fn parse_usize(val: Option<&str>) -> ZpoolResult<usize> {
    let val_str = val.ok_or(ZpoolError::ParseError)?;
    Ok(val_str.parse()?)
}
fn parse_i64(val: Option<&str>) -> ZpoolResult<i64> {
    let val_str = val.ok_or(ZpoolError::ParseError)?;
    Ok(val_str.parse()?)
}
fn parse_string(val: Option<&str>) -> ZpoolResult<String> {
    let val_str = val.ok_or(ZpoolError::ParseError)?;
    Ok(String::from(val_str))
}
impl ZpoolProperties {
    pub fn try_from_stdout(out: &[u8]) -> ZpoolResult<ZpoolProperties> {
            let mut stdout: String = String::from_utf8_lossy(&out).into();
            println!("{:?}", stdout);
            // remove new linr
            stdout.pop();
            let mut cols = stdout.split("\t");

            let alloc = parse_usize(cols.next())?;

            let cap_str = cols.next().ok_or(ZpoolError::ParseError)?;
            let cap: u8 = cap_str.parse()?;

            let comment_str = cols.next().ok_or(ZpoolError::ParseError)?;
            let comment = match comment_str {
                "-" => None,
                 c  => Some(String::from(c))
            };

            let mut dedup_ratio_string = cols.next().ok_or(ZpoolError::ParseError).map(String::from)?;
            // remove 'x'
            dedup_ratio_string.pop();
            let dedup_ratio: f64 = dedup_ratio_string.parse()?;

            let expand_size_str = cols.next().ok_or(ZpoolError::ParseError)?;
            let expand_size: Option<usize> = match expand_size_str {
                "-" => None,
                c   => Some(c.parse()?)
            };

            let mut frag_string = cols.next().ok_or(ZpoolError::ParseError).map(String::from)?;
            // remove 'x'
            frag_string.pop();
            let fragmentation: i8  = frag_string.parse()?;

            let free = parse_i64(cols.next())?;
            let freeing = parse_i64(cols.next())?;
            let guid = parse_string(cols.next())?;
            let health = Health::try_from_str(cols.next())?;
            let size = parse_usize(cols.next())?;
            let leaked = parse_usize(cols.next())?;

            let alt_root_str = cols.next().ok_or(ZpoolError::ParseError)?;
            let alt_root = match alt_root_str {
                "-" => None,
                r   => Some(PathBuf::from(r))
            };


            let read_only = parse_bool(cols.next())?;
            let auto_expand = parse_bool(cols.next())?;
            let auto_replace = parse_bool(cols.next())?;

            let boot_fs_str = cols.next().ok_or(ZpoolError::ParseError)?;
            let boot_fs = match boot_fs_str {
                "-" => None,
                r   => Some(String::from(r))
            };
            let cache_file = CacheType::try_from_str(cols.next())?;
            let dedup_ditto = parse_usize(cols.next())?;
            let delegation = parse_bool(cols.next())?;
            let fail_mode = FailMode::try_from_str(cols.next())?;

            Ok(ZpoolProperties {
                alloc: alloc,
                capacity: cap,
                comment: comment,
                dedup_ratio: dedup_ratio,
                expand_size: expand_size,
                fragmentation: fragmentation,
                free: free,
                freeing: freeing,
                guid: guid,
                health: health,
                size: size,
                leaked: leaked,
                alt_root: alt_root,
                read_only: read_only,
                auto_expand: auto_expand,
                auto_replace: auto_replace,
                boot_fs: boot_fs,
                cache_file: cache_file,
                dedup_ditto: dedup_ditto,
                delegation: delegation,
                fail_mode: fail_mode
            })
    }
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

    #[test]
    fn parsing_health() {
        let online = Some("ONLINE");
        let degraded = Some("DEGRADED");
        let faulted = Some("FAULTED");
        let offline = Some("OFFLINE");
        let unavailable= Some("UNAVAIL");
        let removed = Some("REMOVED");
        let bad = Some("wat");

        assert_eq!(Health::Online, Health::try_from_str(online).unwrap());
        assert_eq!(Health::Degraded, Health::try_from_str(degraded).unwrap());
        assert_eq!(Health::Faulted, Health::try_from_str(faulted).unwrap());
        assert_eq!(Health::Offline, Health::try_from_str(offline).unwrap());
        assert_eq!(Health::Unavailable, Health::try_from_str(unavailable).unwrap());
        assert_eq!(Health::Removed, Health::try_from_str(removed).unwrap());

        let err = Health::try_from_str(bad);
        assert!(err.is_err());

        let err = Health::try_from_str(None);

        assert!(err.is_err());
    }

    #[test]
    fn parsing_fail_mode() {
        let wait = Some("wait");
        let cont = Some("continue");
        let panic = Some("panic");
        let bad = Some("wat");

        assert_eq!(FailMode::Wait, FailMode::try_from_str(wait).unwrap());
        assert_eq!(FailMode::Continue, FailMode::try_from_str(cont).unwrap());
        assert_eq!(FailMode::Panic, FailMode::try_from_str(panic).unwrap());

        let err = FailMode::try_from_str(bad);
        assert!(err.is_err());

        let err = FailMode::try_from_str(None);
        assert!(err.is_err());
    }

    #[test]
    fn parsing_cache_file() {
        assert_eq!(CacheType::Default, CacheType::try_from_str(Some("-")).unwrap());
        assert_eq!(CacheType::Default, CacheType::try_from_str(Some("")).unwrap());
        assert_eq!(CacheType::None, CacheType::try_from_str(Some("none")).unwrap());
        assert_eq!(CacheType::Custom("/wat".into()), CacheType::try_from_str(Some("/wat")).unwrap());

        let err = CacheType::try_from_str(None);
        assert!(err.is_err());
    }

    #[test]
    fn parsing_props_i128_guid() {
        let line = b"69120\t0\t-\t1.00x\t-\t1%\t67039744\t0\t15867762423891129245\tONLINE\t67108864\t0\t-\toff\toff\toff\t-\t-\t0\ton\twait\n";
        let props = ZpoolProperties::try_from_stdout(line);
        assert!(props.is_ok());
    }

    #[test]
    fn parsing_props() {
        let line = b"69120\t0\t-\t1.50x\t-\t22%\t67039744\t0\t4957928072935098740\tONLINE\t67108864\t0\t-\toff\toff\toff\t-\t-\t0\ton\twait\n";
        let props = ZpoolProperties::try_from_stdout(line);
        assert!(props.is_ok());

        let line = b"69120\t0\ttouch it\t1.50x\t-\t22%\t67039744\t0\t4957928072935098740\tONLINE\t67108864\t0\t-\toff\toff\toff\t-\t-\t0\ton\tpanic\n";
        let props = ZpoolProperties::try_from_stdout(line).unwrap();
        assert_eq!(Some(String::from("touch it")), props.comment);
        assert_eq!(FailMode::Panic, props.fail_mode);

        let line = b"69120\t0\ttouch it\t1.50x\t-\t22%\t67039744\t0\t4957928072935098740\tOFFLINE\t67108864\t0\t/mnt/\toff\toff\toff\t-\t-\t0\ton\twait\n";
        let props = ZpoolProperties::try_from_stdout(line).unwrap();
        assert_eq!(Health::Offline, props.health);
        assert_eq!(Some(PathBuf::from("/mnt")), props.alt_root);

        let line = b"waf\tasd";
        let props = ZpoolProperties::try_from_stdout(line);
        assert!(props.is_err());
    }
}
