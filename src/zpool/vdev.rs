/// CreateVdevRequest data types
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// Every vdev can be backed either by a block device or a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Disk {
    /// Sparse file based device.
    File(PathBuf),
    /// Block device.
    Disk(PathBuf),
}

impl Disk {
    /// Verify that disk is valid. Just because it valid doesn't mean zpool can
    /// use it.
    /// all it does it verifies that path exists. For now they both look the
    /// same. Distinction exists to make sure it will work in the future.
    pub fn is_valid(&self) -> bool {
        match *self {
            Disk::File(ref path) => path.exists(),
            Disk::Disk(ref path) => Path::new("/dev/").join(path).exists()
        }
    }

    /// Make Disk usable as arg for Command.
    pub fn into_arg(self) -> OsString {
        match self {
            Disk::File(path) | Disk::Disk(path) => path.into_os_string(),
        }
    }

    /// Make Disk usable as arg for Command.
    pub fn as_arg(&self) -> &OsStr {
        match self {
            Disk::File(path) | Disk::Disk(path) => path.as_os_str(),
        }
    }

    /// Make a reference to a block device.
    pub fn disk<O: Into<PathBuf>>(value: O) -> Disk { Disk::Disk(value.into()) }

    /// Make a reference to a sparse file.
    pub fn file<O: Into<PathBuf>>(value: O) -> Disk { Disk::File(value.into()) }
}

/// Basic building block of
/// [Zpool](https://www.freebsd.org/doc/handbook/zfs-term.html).
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

impl VdevType {
    pub fn from_str(source: &str) -> VdevType {
        match source {
            "mirror" => VdevType::Mirror,
            "raidz1" => VdevType::RaidZ,
            "raidz2" => VdevType::RaidZ2,
            "raidz3" => VdevType::RaidZ3,
            _ => VdevType::SingleDisk
        }
    }
}

/// Basic building block of
/// [Zpool](https://www.freebsd.org/doc/handbook/zfs-term.html).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateVdevRequest {
    /// Just a single disk or file.
    SingleDisk(PathBuf),
    /// A mirror of multiple vdevs
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
        return true;
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
            CreateVdevRequest::SingleDisk(ref disk) => true,
            CreateVdevRequest::Mirror(ref disks) => CreateVdevRequest::is_valid_raid(disks, 2),
            CreateVdevRequest::RaidZ(ref disks) => CreateVdevRequest::is_valid_raid(disks, 3),
            CreateVdevRequest::RaidZ2(ref disks) => CreateVdevRequest::is_valid_raid(disks, 5),
            CreateVdevRequest::RaidZ3(ref disks) => CreateVdevRequest::is_valid_raid(disks, 8),
        }
    }

    #[inline]
    fn conv_to_args<T: Into<OsString>>(vdev_type: T, disks: Vec<PathBuf>) -> Vec<OsString> {
        let mut ret = Vec::with_capacity(disks.len() + 1);
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
    pub fn disk<O: Into<PathBuf>>(value: O) -> CreateVdevRequest { CreateVdevRequest::SingleDisk(value.into()) }
}

/// A pool is made up of one or more vdevs, which themselves can be a single disk or a group
/// of disks, in the case of a RAID transform. When multiple vdevs are used, ZFS spreads data
/// across the vdevs to increase performance and maximize usable space.
pub struct Vdev {
    /// Type of Vdev
    kind: VdevType,
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
    fn test_disk_validation() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();
        let invalid_path = tmp_dir.path().join("fake");

        let valid_disk_file = Disk::File(file_path.clone());
        let invalid_disk_file = Disk::File(invalid_path.clone());
        assert!(valid_disk_file.is_valid());
        assert!(!invalid_disk_file.is_valid());

        let valid_disk = Disk::Disk(file_path.clone());
        let invalid_disk = Disk::Disk(invalid_path.clone());
        assert!(valid_disk.is_valid());
        assert!(!invalid_disk.is_valid());
    }

    #[test]
    fn test_raid_validation_naked() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();
        let invalid_path = tmp_dir.path().join("fake");

        let vdev = CreateVdevRequest::SingleDisk(file_path);
        let invalid_vdev = CreateVdevRequest::SingleDisk(invalid_path);
        assert!(vdev.is_valid());
        assert!(!invalid_vdev.is_valid());
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
        let expected: Vec<OsString> = vec![
            "mirror".into(),
            file_path.clone().into(),
            file_path.clone().into(),
        ];
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
}
