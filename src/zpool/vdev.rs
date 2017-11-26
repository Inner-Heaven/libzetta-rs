/// Vdec data types

use std::ffi::OsString;
use std::path::PathBuf;

/// Every vdev can be backed either by block device or sparse file.
#[derive(Debug, Clone, PartialEq)]
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
            Disk::File(ref path) |
            Disk::Disk(ref path) => path.exists(),
        }
    }

    /// Make Disk usable as arg for Command.
    pub fn into_arg(self) -> OsString {
        match self {
            Disk::File(path) | Disk::Disk(path) => path.into_os_string(),
        }
    }

    /// Make a reference to a block device.
    pub fn disk<O: Into<PathBuf>>(value: O) -> Disk { Disk::Disk(value.into()) }

    /// Make a reference to a sparse file.
    pub fn file<O: Into<PathBuf>>(value: O) -> Disk { Disk::File(value.into()) }
}

/// Basic building block of
/// [Zpool](https://www.freebsd.org/doc/handbook/zfs-term.html).
#[derive(Debug, Clone, PartialEq)]
pub enum Vdev {
    /// Just a single disk or file.
    Naked(Disk),
    /// A mirror of multiple vdevs
    Mirror(Vec<Disk>),
    /// ZFS implements [RAID-Z](https://blogs.oracle.com/ahl/what-is-raid-z), a
    /// variation on standard RAID-5 that offers better distribution of
    /// parity and eliminates the “RAID-5 write hole”.
    RaidZ(Vec<Disk>),
    /// The same as RAID-Z, but with 2 parity drives.
    RaidZ2(Vec<Disk>),
    /// The same as RAID-Z, but with 3 parity drives.
    RaidZ3(Vec<Disk>),
}

impl Vdev {
    #[inline]
    fn is_valid_raid(disks: &[Disk], min_disks: usize) -> bool {
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
    /// This gives false negative results in RAIDZ2 and RAIDZ3. This is
    /// intentional.
    /// possible makes no sense.
    pub fn is_valid(&self) -> bool {
        match *self {
            Vdev::Naked(ref disk) => disk.is_valid(),
            Vdev::Mirror(ref disks) => Vdev::is_valid_raid(disks, 2),
            Vdev::RaidZ(ref disks) => Vdev::is_valid_raid(disks, 3),
            Vdev::RaidZ2(ref disks) => Vdev::is_valid_raid(disks, 5),
            Vdev::RaidZ3(ref disks) => Vdev::is_valid_raid(disks, 8),
        }
    }

    #[inline]
    fn conv_to_args<T: Into<OsString>>(vdev_type: T, disks: Vec<Disk>) -> Vec<OsString> {
        let mut ret = Vec::with_capacity(disks.len() + 1);
        ret.push(vdev_type.into());
        for disk in disks {
            ret.push(disk.into_arg());
        }
        ret
    }
    /// Make turn Vdev into list of arguments.
    pub fn into_args(self) -> Vec<OsString> {
        match self {
            Vdev::Naked(disk) => vec![disk.into_arg()],
            Vdev::Mirror(disks) => Vdev::conv_to_args("mirror", disks),
            Vdev::RaidZ(disks) => Vdev::conv_to_args("raidz", disks),
            Vdev::RaidZ2(disks) => Vdev::conv_to_args("raidz2", disks),
            Vdev::RaidZ3(disks) => Vdev::conv_to_args("raidz3", disks),
        }
    }
    /// Short-cut to Vdev::Naked(Disk::Disk(disk))
    pub fn disk<O: Into<PathBuf>>(value: O) -> Vdev { Vdev::Naked(Disk::Disk(value.into())) }

    /// Short-cut to Vdev::Naked(Disk::File(disk))
    pub fn file<O: Into<PathBuf>>(value: O) -> Vdev { Vdev::Naked(Disk::File(value.into())) }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::fs::File;
    use tempdir::TempDir;

    fn get_disks(num: usize, path: &PathBuf) -> Vec<Disk> {
        (0..num).map(|_| Disk::File(path.clone())).collect()
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

        let vdev = Vdev::Naked(Disk::File(file_path));
        let invalid_vdev = Vdev::Naked(Disk::File(invalid_path));
        assert!(vdev.is_valid());
        assert!(!invalid_vdev.is_valid());
    }

    #[test]
    fn test_raid_validation_mirror() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();

        let vdev = Vdev::Mirror(get_disks(2, &file_path));
        assert!(vdev.is_valid());

        let bad = Vdev::Mirror(get_disks(1, &file_path));
        assert!(!bad.is_valid());

        let also_bad = Vdev::Mirror(get_disks(0, &file_path));
        assert!(!also_bad.is_valid());
    }

    #[test]
    fn test_raid_validation_raidz() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();

        let vdev = Vdev::RaidZ(get_disks(3, &file_path));
        assert!(vdev.is_valid());

        let also_vdev = Vdev::RaidZ(get_disks(5, &file_path));
        assert!(also_vdev.is_valid());

        let bad = Vdev::RaidZ(get_disks(2, &file_path));
        assert!(!bad.is_valid());

        let also_bad = Vdev::RaidZ(get_disks(1, &file_path));
        assert!(!also_bad.is_valid());
    }

    #[test]
    fn test_raid_validation_raidz2() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();

        let vdev = Vdev::RaidZ2(get_disks(5, &file_path));
        assert!(vdev.is_valid());

        let also_vdev = Vdev::RaidZ2(get_disks(8, &file_path));
        assert!(also_vdev.is_valid());

        let bad = Vdev::RaidZ2(get_disks(3, &file_path));
        assert!(!bad.is_valid());

        let also_bad = Vdev::RaidZ2(get_disks(1, &file_path));
        assert!(!also_bad.is_valid());
    }

    #[test]
    fn test_raid_validation_raidz3() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();

        let vdev = Vdev::RaidZ3(get_disks(8, &file_path));
        assert!(vdev.is_valid());

        let also_vdev = Vdev::RaidZ3(get_disks(10, &file_path));
        assert!(also_vdev.is_valid());

        let bad = Vdev::RaidZ3(get_disks(3, &file_path));
        assert!(!bad.is_valid());

        let also_bad = Vdev::RaidZ3(get_disks(0, &file_path));
        assert!(!also_bad.is_valid());
    }

    #[test]
    fn test_vdev_to_arg_naked() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();

        let vdev = Vdev::Naked(Disk::File(file_path.clone()));

        let args = vdev.into_args();
        assert_eq!(vec![file_path], args);
    }
    #[test]
    fn test_vdev_to_arg_mirror() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();

        let vdev = Vdev::Mirror(get_disks(2, &file_path));

        let args = vdev.into_args();
        let expected: Vec<OsString> = vec!["mirror".into(),
                                           file_path.clone().into(),
                                           file_path.clone().into()];
        assert_eq!(expected, args);
    }

    #[test]
    fn test_vdev_to_arg_raidz() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();

        let vdev = Vdev::RaidZ(get_disks(3, &file_path));

        let args = vdev.into_args();
        assert_eq!(4, args.len());
        assert_eq!(OsString::from("raidz"), args[0]);
    }

    #[test]
    fn test_vdev_to_arg_raidz2() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();

        let vdev = Vdev::RaidZ2(get_disks(5, &file_path));

        let args = vdev.into_args();
        assert_eq!(6, args.len());
        assert_eq!(OsString::from("raidz2"), args[0]);
    }
    #[test]
    fn test_vdev_to_arg_raidz3() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();

        let vdev = Vdev::RaidZ3(get_disks(8, &file_path));

        let args = vdev.into_args();
        assert_eq!(9, args.len());
        assert_eq!(OsString::from("raidz3"), args[0]);
    }

    #[test]
    fn short_versions_disk() {
        let name = "wat";
        let path = PathBuf::from(&name);
        let disk = Vdev::Naked(Disk::disk(path.clone()));
        let disk_left = Vdev::Naked(Disk::Disk(path.clone()));

        assert_eq!(disk_left, disk);

        assert_eq!(disk_left, Vdev::disk(name.clone()));
    }

    #[test]
    fn short_versions_file() {
        let name = "wat";
        let path = PathBuf::from(&name);
        let file = Vdev::Naked(Disk::file(path.clone()));
        let file_left = Vdev::Naked(Disk::File(path.clone()));

        assert_eq!(file_left, file);

        assert_eq!(file_left, Vdev::file(name.clone()));
    }
}
