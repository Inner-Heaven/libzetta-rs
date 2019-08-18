//! Consumer friendly structure representing vdev.
//!
//! Everything that goes into vdev is in this module.
//!
//! ### Examples
//!
//! ##### Create a mirror with 2 disks
//!
//! ```rust
//! use libzetta::zpool::CreateVdevRequest;
//! use std::path::PathBuf;
//!
//! // Create an `Vec` with two disks
//! let drives = vec![PathBuf::from("nvd0p4.eli"), PathBuf::from("nvd1p4.eli")];
//! let vdev = CreateVdevRequest::Mirror(drives);
//! ```
//! ##### Create a single disk vdev with sparse file
//!
//! ```rust
//! use libzetta::zpool::CreateVdevRequest;
//! use std::path::PathBuf;
//! // (file needs to exist prior)
//! let path = PathBuf::from("/tmp/sparseFile0");
//! let vdev = CreateVdevRequest::SingleDisk(path);
//! ```

use std::{default::Default,
          ffi::OsString,
          path::{Path, PathBuf},
          str::FromStr};

use crate::zpool::{Health, Reason, ZpoolError};

/// Error statistics.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ErrorStatistics {
    /// I/O errors that occurred while issuing a read request
    pub read: u64,
    /// I/O errors that occurred while issuing a write request
    pub write: u64,
    /// Checksum errors, meaning that the device returned corrupted data as the
    /// result of a read request
    pub checksum: u64,
}

impl Default for ErrorStatistics {
    fn default() -> ErrorStatistics { ErrorStatistics { read: 0, write: 0, checksum: 0 } }
}

/// Basic building block of vdev.
///
/// It can be backed by a entire block device, a partition or a file. This particular structure
/// represents backing of existing vdev. If disk is part of active zpool then it will also
/// have error counts.
#[derive(Debug, Clone, Getters, Eq, Builder)]
#[builder(setter(into))]
#[get = "pub"]
pub struct Disk {
    /// Path to a backing device or file. If path is relative, then it's
    /// relative to `/dev/`.
    path: PathBuf,
    /// Current health of this specific device.
    health: Health,
    /// Reason why device is in this state.
    #[builder(default)]
    reason: Option<Reason>,
    /// How many read, write and checksum errors device encountered since last
    /// reset.
    #[builder(default)]
    error_statistics: ErrorStatistics,
}

impl Disk {
    pub fn builder() -> DiskBuilder { DiskBuilder::default() }
}

/// Equal if path is the same.
impl PartialEq for Disk {
    fn eq(&self, other: &Disk) -> bool { self.path == other.path }
}

impl PartialEq<Path> for Disk {
    fn eq(&self, other: &Path) -> bool { self.path.as_path() == other }
}

impl PartialEq<PathBuf> for Disk {
    fn eq(&self, other: &PathBuf) -> bool { &self.path == other }
}

impl PartialEq<Disk> for PathBuf {
    fn eq(&self, other: &Disk) -> bool { other == self }
}

impl PartialEq<Disk> for Path {
    fn eq(&self, other: &Disk) -> bool { other == self }
}

/// A [type](https://www.freebsd.org/doc/handbook/zfs-term.html) of Vdev.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VdevType {
    /// Just a single disk or file.
    SingleDisk,
    /// A mirror of multiple vdevs
    Mirror,
    /// ZFS implements [RAID-Z](https://blogs.oracle.com/ahl/what-is-raid-z), a
    /// variation on standard RAID-5 that offers better distribution of
    /// parity and eliminates the “RAID-5 write hole”.
    RaidZ,
    /// The same as RAID-Z, but with 2 parity drives.
    RaidZ2,
    /// The same as RAID-Z, but with 3 parity drives.
    RaidZ3,
}

impl FromStr for VdevType {
    type Err = ZpoolError;

    fn from_str(source: &str) -> Result<VdevType, ZpoolError> {
        match source {
            "mirror" => Ok(VdevType::Mirror),
            "raidz1" => Ok(VdevType::RaidZ),
            "raidz2" => Ok(VdevType::RaidZ2),
            "raidz3" => Ok(VdevType::RaidZ3),
            n => Err(ZpoolError::UnknownRaidType(String::from(n))),
        }
    }
}

/// Consumer friendly wrapper to configure vdev to zpol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateVdevRequest {
    /// The most basic type of vdev is a standard block device. This can be an
    /// entire disk or a partition. In addition to disks, ZFS pools can be
    /// backed by regular files, this is especially useful for testing and
    /// experimentation. Use the full path to the file as the device path in
    /// zpool create. All vdevs must be at least 64MB or 128 MB in size
    /// depending on implementation.
    SingleDisk(PathBuf),
    /// A mirror of multiple disks. A mirror vdev will only hold as much data as
    /// its smallest member. A mirror vdev can withstand the failure of all
    /// but one of its members without losing any data.
    Mirror(Vec<PathBuf>),
    /// ZFS implements [RAID-Z](https://blogs.oracle.com/ahl/what-is-raid-z), a
    /// variation on standard RAID-5 that offers better distribution of
    /// parity and eliminates the “RAID-5 write hole”.
    RaidZ(Vec<PathBuf>),
    /// The same as RAID-Z, but with 2 parity drives.
    RaidZ2(Vec<PathBuf>),
    /// The same as RAID-Z, but with 3 parity drives.
    RaidZ3(Vec<PathBuf>),
}

impl CreateVdevRequest {
    #[inline]
    fn is_valid_raid(disks: &[PathBuf], min_disks: usize) -> bool {
        if disks.len() < min_disks {
            return false;
        }
        true
    }

    /// Check if given CreateVdevRequest is valid.
    ///
    /// For SingleDisk it means that what ever it points to exists.
    ///
    /// For Mirror it checks that it's at least two valid disks.
    ///
    /// For RaidZ it checks that it's at least three valid disk. And so goes on.
    /// This gives false negative results in RAIDZ2 and RAIDZ3. This is
    /// intentional.
    /// possible makes no sense.
    pub fn is_valid(&self) -> bool {
        match *self {
            CreateVdevRequest::SingleDisk(ref _disk) => true,
            CreateVdevRequest::Mirror(ref disks) => CreateVdevRequest::is_valid_raid(disks, 2),
            CreateVdevRequest::RaidZ(ref disks) => CreateVdevRequest::is_valid_raid(disks, 3),
            CreateVdevRequest::RaidZ2(ref disks) => CreateVdevRequest::is_valid_raid(disks, 5),
            CreateVdevRequest::RaidZ3(ref disks) => CreateVdevRequest::is_valid_raid(disks, 8),
        }
    }

    #[inline]
    fn conv_to_args<T: Into<OsString>>(vdev_type: T, disks: Vec<PathBuf>) -> Vec<OsString> {
        let mut ret = Vec::with_capacity(disks.len());
        ret.push(vdev_type.into());
        for disk in disks {
            ret.push(disk.into_os_string());
        }
        ret
    }

    /// Make turn CreateVdevRequest into list of arguments.
    pub fn into_args(self) -> Vec<OsString> {
        match self {
            CreateVdevRequest::SingleDisk(disk) => vec![disk.into_os_string()],
            CreateVdevRequest::Mirror(disks) => CreateVdevRequest::conv_to_args("mirror", disks),
            CreateVdevRequest::RaidZ(disks) => CreateVdevRequest::conv_to_args("raidz", disks),
            CreateVdevRequest::RaidZ2(disks) => CreateVdevRequest::conv_to_args("raidz2", disks),
            CreateVdevRequest::RaidZ3(disks) => CreateVdevRequest::conv_to_args("raidz3", disks),
        }
    }

    /// Short-cut to CreateVdevRequest::SingleDisk(disk)
    pub fn disk<O: Into<PathBuf>>(value: O) -> CreateVdevRequest {
        CreateVdevRequest::SingleDisk(value.into())
    }

    /// Get kind
    pub fn kind(&self) -> VdevType {
        match self {
            CreateVdevRequest::SingleDisk(_) => VdevType::SingleDisk,
            CreateVdevRequest::Mirror(_) => VdevType::Mirror,
            CreateVdevRequest::RaidZ(_) => VdevType::RaidZ,
            CreateVdevRequest::RaidZ2(_) => VdevType::RaidZ2,
            CreateVdevRequest::RaidZ3(_) => VdevType::RaidZ3,
        }
    }
}

impl PartialEq<Vdev> for CreateVdevRequest {
    fn eq(&self, other: &Vdev) -> bool { other == self }
}

/// Basic zpool building block.
///
/// A pool is made up of one or more vdevs, which themselves can be a single
/// disk or a group of disks, in the case of a RAID transform. When multiple
/// vdevs are used, ZFS spreads data across the vdevs to increase performance
/// and maximize usable space.
#[derive(Debug, Clone, Getters, Builder, Eq)]
#[get = "pub"]
pub struct Vdev {
    /// Type of Vdev
    kind: VdevType,
    /// Current Health of Vdev
    health: Health,
    /// Reason why vdev is in this state
    #[builder(default)]
    reason: Option<Reason>,
    /// Backing devices for this vdev
    disks: Vec<Disk>,
    /// How many read, write and checksum errors device encountered since last
    /// reset.
    #[builder(default)]
    error_statistics: ErrorStatistics,
}

impl Vdev {
    /// Create a builder - a referred way of creating Vdev structure.
    pub fn builder() -> VdevBuilder { VdevBuilder::default() }
}
/// Vdevs are equal of their type and backing disks are equal.
impl PartialEq for Vdev {
    fn eq(&self, other: &Vdev) -> bool {
        self.kind() == other.kind() && self.disks() == other.disks()
    }
}

impl PartialEq<CreateVdevRequest> for Vdev {
    fn eq(&self, other: &CreateVdevRequest) -> bool {
        self.kind() == &other.kind() && {
            match other {
                CreateVdevRequest::SingleDisk(ref d) => {
                    self.disks().first().map(Disk::path) == Some(d)
                },
                CreateVdevRequest::Mirror(ref disks) => self.disks() == disks,
                CreateVdevRequest::RaidZ(ref disks) => self.disks() == disks,
                CreateVdevRequest::RaidZ2(ref disks) => self.disks() == disks,
                CreateVdevRequest::RaidZ3(ref disks) => self.disks() == disks,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;

    use tempdir::TempDir;

    use super::*;

    fn get_disks(num: usize, path: &PathBuf) -> Vec<PathBuf> {
        (0..num).map(|_| path.clone()).collect()
    }

    #[test]
    fn test_raid_validation_naked() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");

        let vdev = CreateVdevRequest::SingleDisk(file_path);
        assert!(vdev.is_valid());
    }

    #[test]
    fn test_raid_validation_mirror() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();

        let vdev = CreateVdevRequest::Mirror(get_disks(2, &file_path));
        assert!(vdev.is_valid());

        let bad = CreateVdevRequest::Mirror(get_disks(1, &file_path));
        assert!(!bad.is_valid());

        let also_bad = CreateVdevRequest::Mirror(get_disks(0, &file_path));
        assert!(!also_bad.is_valid());
    }

    #[test]
    fn test_raid_validation_raidz() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();

        let vdev = CreateVdevRequest::RaidZ(get_disks(3, &file_path));
        assert!(vdev.is_valid());

        let also_vdev = CreateVdevRequest::RaidZ(get_disks(5, &file_path));
        assert!(also_vdev.is_valid());

        let bad = CreateVdevRequest::RaidZ(get_disks(2, &file_path));
        assert!(!bad.is_valid());

        let also_bad = CreateVdevRequest::RaidZ(get_disks(1, &file_path));
        assert!(!also_bad.is_valid());
    }

    #[test]
    fn test_raid_validation_raidz2() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();

        let vdev = CreateVdevRequest::RaidZ2(get_disks(5, &file_path));
        assert!(vdev.is_valid());

        let also_vdev = CreateVdevRequest::RaidZ2(get_disks(8, &file_path));
        assert!(also_vdev.is_valid());

        let bad = CreateVdevRequest::RaidZ2(get_disks(3, &file_path));
        assert!(!bad.is_valid());

        let also_bad = CreateVdevRequest::RaidZ2(get_disks(1, &file_path));
        assert!(!also_bad.is_valid());
    }

    #[test]
    fn test_raid_validation_raidz3() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();

        let vdev = CreateVdevRequest::RaidZ3(get_disks(8, &file_path));
        assert!(vdev.is_valid());

        let also_vdev = CreateVdevRequest::RaidZ3(get_disks(10, &file_path));
        assert!(also_vdev.is_valid());

        let bad = CreateVdevRequest::RaidZ3(get_disks(3, &file_path));
        assert!(!bad.is_valid());

        let also_bad = CreateVdevRequest::RaidZ3(get_disks(0, &file_path));
        assert!(!also_bad.is_valid());
    }

    #[test]
    fn test_vdev_to_arg_naked() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();

        let vdev = CreateVdevRequest::SingleDisk(file_path.clone());

        let args = vdev.into_args();
        assert_eq!(vec![file_path], args);
    }
    #[test]
    fn test_vdev_to_arg_mirror() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();

        let vdev = CreateVdevRequest::Mirror(get_disks(2, &file_path));

        let args = vdev.into_args();
        let expected: Vec<OsString> =
            vec!["mirror".into(), file_path.clone().into(), file_path.clone().into()];
        assert_eq!(expected, args);
    }

    #[test]
    fn test_vdev_to_arg_raidz() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();

        let vdev = CreateVdevRequest::RaidZ(get_disks(3, &file_path));

        let args = vdev.into_args();
        assert_eq!(4, args.len());
        assert_eq!(OsString::from("raidz"), args[0]);
    }

    #[test]
    fn test_vdev_to_arg_raidz2() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();

        let vdev = CreateVdevRequest::RaidZ2(get_disks(5, &file_path));

        let args = vdev.into_args();
        assert_eq!(6, args.len());
        assert_eq!(OsString::from("raidz2"), args[0]);
    }
    #[test]
    fn test_vdev_to_arg_raidz3() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();

        let vdev = CreateVdevRequest::RaidZ3(get_disks(8, &file_path));

        let args = vdev.into_args();
        assert_eq!(9, args.len());
        assert_eq!(OsString::from("raidz3"), args[0]);
    }

    #[test]
    fn short_versions_disk() {
        let name = "wat";
        let path = PathBuf::from(&name);
        let disk = CreateVdevRequest::SingleDisk(path.clone());
        let disk_left = CreateVdevRequest::SingleDisk(path.clone());

        assert_eq!(disk_left, disk);

        assert_eq!(disk_left, CreateVdevRequest::disk(name.clone()));
    }

    #[test]
    fn test_path_eq_disk() {
        let path = PathBuf::from("wat");
        let disk = Disk::builder().path("wat").health(Health::Online).build().unwrap();
        assert_eq!(path, disk);
        assert_eq!(path.as_path(), &disk);
        assert_eq!(disk, path);
        assert_eq!(&disk, path.as_path());
    }

    #[test]
    fn test_path_ne_disk() {
        let path = PathBuf::from("wat");
        let disk = Disk::builder().path("notwat").health(Health::Online).build().unwrap();
        assert_ne!(path, disk);
        assert_ne!(path.as_path(), &disk);
        assert_ne!(disk, path);
        assert_ne!(&disk, path.as_path());
    }

    #[test]
    fn test_vdev_eq_vdev() {
        let disk = Disk::builder().path("notwat").health(Health::Online).build().unwrap();

        let left = Vdev::builder()
            .kind(VdevType::SingleDisk)
            .health(Health::Online)
            .disks(vec![disk.clone()])
            .build()
            .unwrap();
        assert_eq!(left, left.clone());
    }

    #[test]
    fn test_vdev_ne_vdev() {
        let disk = Disk::builder().path("notwat").health(Health::Online).build().unwrap();

        let left = Vdev::builder()
            .kind(VdevType::SingleDisk)
            .health(Health::Online)
            .disks(vec![disk.clone()])
            .build()
            .unwrap();

        let right = Vdev::builder()
            .kind(VdevType::RaidZ)
            .health(Health::Online)
            .disks(vec![disk.clone()])
            .build()
            .unwrap();

        assert_ne!(left, right);

        let disk2 = Disk::builder().path("wat").health(Health::Online).build().unwrap();
        let right = Vdev::builder()
            .kind(VdevType::RaidZ)
            .health(Health::Online)
            .disks(vec![disk2])
            .build()
            .unwrap();

        assert_ne!(left, right);
    }
}
