/// CreateZpoolRequest is a structure that describes zpool vdev structure.
/// Use to create and updated zpool
use std::ffi::OsString;
use std::path::PathBuf;

use zpool::CreateMode;
use zpool::properties::ZpoolPropertiesWrite;
use zpool::vdev::CreateVdevRequest;

/// Structure representing what zpool consist of.
/// This structure is used in zpool creation and when new drives are attached.
///
/// ### Examples
///
/// Let's create simple topology: 2 drives in mirror, no l2arc, no zil.
///
/// ```rust
/// use libzfs::zpool::CreateZpoolRequestBuilder;
/// use libzfs::zpool::{Disk, CreateVdevRequest};
///
/// let drives = vec![Disk::disk("hd0"), Disk::disk("hd1")];
/// let topo = CreateZpoolRequestBuilder::default()
///     .name("tank")
///     .vdevs(vec![CreateVdevRequest::Mirror(drives)])
///     .build()
///     .unwrap();
/// ```
/// Overkill example: 2 drives in mirror and a single drive, zil on double
/// mirror and 2 l2rc.
///
/// ```rust
/// use libzfs::zpool::CreateZpoolRequestBuilder;
/// use libzfs::zpool::{Disk, CreateVdevRequest};
///
/// let zil_drives = vec![Disk::Disk("hd0".into()), Disk::Disk("hd1".into())];
/// let mirror_drives = vec![Disk::Disk("hd2".into()), Disk::Disk("hd3".into())];
/// let cache_drives = vec![Disk::Disk("hd4".into()), Disk::Disk("hd5".into())];
/// let topo = CreateZpoolRequestBuilder::default()
///     .name("tank")
///     .vdevs(vec![CreateVdevRequest::Mirror(mirror_drives)])
///     .cache(Disk::File("/tmp/sparse.file".into()))
///     .vdev(CreateVdevRequest::SingleDisk(Disk::Disk("sd0".into())))
///     .caches(cache_drives)
///     .zil(CreateVdevRequest::Mirror(zil_drives))
///     .altroot(std::path::PathBuf::new("/mnt")
///     .mount(std::path::PathBuf::new("/mnt")
///     .build()
///     .unwrap();
/// ```
#[derive(Default, Builder, Debug, Clone, Getters, PartialEq, Eq)]
#[builder(setter(into))]
pub struct CreateZpoolRequest {
    /// Name to give new zpool
    #[builder(default = "String::from(\"tank\")")]
    name: String,
    /// Properties if new zpool
    #[builder(default)]
    props: Option<ZpoolPropertiesWrite>,
    /// Altroot for zpool
    #[builder(default)]
    altroot: Option<PathBuf>,
    /// Mount mount point for zpool
    #[builder(default)]
    mount: Option<PathBuf>,
    /// Use `-f` or not;
    #[builder(default)]
    create_mode: CreateMode,
    /// Devices used to store data
    #[builder(default)]
    vdevs: Vec<CreateVdevRequest>,
    /// Adding a cache vdev to a pool will add the storage of the cache to the
    /// [L2ARC](https://www.freebsd.org/doc/handbook/zfs-term.html#zfs-term-l2arc). Cache devices
    /// cannot be mirrored. Since a cache device only stores additional copies of existing data,
    /// there is no risk of data loss.
    #[builder(default)]
    caches: Vec<PathBuf>,
    /// ZFS Log Devices, also known as ZFS Intent Log ([ZIL](https://www.freebsd.org/doc/handbook/zfs-term.html#zfs-term-zil)) move the intent log from the regular
    /// pool devices to a dedicated device, typically an SSD. Having a dedicated log device can
    /// significantly improve the performance of applications with a high volume of *synchronous*
    /// writes, especially databases. Log devices can be mirrored, but RAID-Z is not supported.
    /// If multiple log devices are used, writes will be load balanced across them
    #[builder(default)]
    zil: Option<CreateVdevRequest>,
}

impl CreateZpoolRequest {
    /// Verify that given topology can be used to update existing pool.
    pub fn is_suitable_for_update(&self) -> bool {
        let valid_vdevs = self.vdevs.iter().all(CreateVdevRequest::is_valid);
        if !valid_vdevs {
            return false;
        }

        match self.zil {
            Some(ref vdev) => vdev.is_valid(),
            None => true,
        }
    }
    /// Verify that given topology can be used to create new zpool.
    ///
    /// That means it as at least one valid vdev and all optional devices are
    /// valid if present.
    pub fn is_suitable_for_create(&self) -> bool {
        if self.vdevs.is_empty() {
            return false;
        }
        self.is_suitable_for_update()
    }

    /// Make CreateZpoolRequest usable as arg for Command
    pub fn into_args(self) -> Vec<OsString> {
        let mut ret: Vec<OsString> = Vec::with_capacity(13);

        let vdevs = self.vdevs.into_iter().flat_map(CreateVdevRequest::into_args);
        let zil = self.zil.map(CreateVdevRequest::into_args);
        ret.extend(vdevs);

        if !self.caches.is_empty() {
            let caches = self.caches.into_iter().map(PathBuf::into_os_string);
            ret.push("cache".into());
            ret.extend(caches);
        }

        if let Some(z) = zil {
            ret.push("log".into());
            ret.extend(z);
        }

        ret
    }
}

impl CreateZpoolRequestBuilder {
    pub fn vdev(&mut self, vdev: CreateVdevRequest) -> &mut CreateZpoolRequestBuilder {
        match self.vdevs {
            Some(ref mut vec) => vec.push(vdev),
            None => {
                self.vdevs = Some(Vec::new());
                return self.vdev(vdev);
            }
        }
        self
    }

    pub fn cache(&mut self, disk: PathBuf) -> &mut CreateZpoolRequestBuilder {
        match self.caches {
            Some(ref mut vec) => vec.push(disk),
            None => {
                self.caches = Some(Vec::new());
                return self.cache(disk);
            }
        }
        self
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::path::PathBuf;

    use tempdir::TempDir;

    use super::*;

    fn get_disks(num: usize, path: &PathBuf) -> Vec<PathBuf> {
        (0..num).map(|_| path.clone()).collect()
    }

    fn args_from_slice(args: &[&str]) -> Vec<OsString> {
        args.to_vec().into_iter().map(OsString::from).collect()
    }

    #[test]
    fn test_validators() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let _valid_file = File::create(file_path.clone()).unwrap();

        // Zpool with one valid mirror
        let topo = CreateZpoolRequestBuilder::default()
            .vdevs(vec![CreateVdevRequest::Mirror(get_disks(2, &file_path))])
            .build()
            .unwrap();

        assert!(topo.is_suitable_for_create());

        // Zpool with invalid mirror
        let topo = CreateZpoolRequestBuilder::default()
            .vdevs(vec![CreateVdevRequest::Mirror(get_disks(1, &file_path))])
            .build()
            .unwrap();

        assert!(!topo.is_suitable_for_create());

        // Zpool with valid cache and valid vdev
        let topo = CreateZpoolRequestBuilder::default()
            .vdevs(vec![CreateVdevRequest::Mirror(get_disks(2, &file_path))])
            .caches(get_disks(2, &file_path))
            .build()
            .unwrap();

        assert!(topo.is_suitable_for_create());

        // Just add L2ARC to zpool
        let topo = CreateZpoolRequestBuilder::default()
            .cache(file_path)
            .build()
            .unwrap();

        assert!(topo.is_suitable_for_update());
        assert!(!topo.is_suitable_for_create());
    }

    #[test]
    fn test_args() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let path = file_path.to_str().unwrap();
        let _valid_file = File::create(file_path.clone()).unwrap();
        let naked_vdev = CreateVdevRequest::SingleDisk(file_path.clone());

        // Just add L2ARC to zpool
        let topo = CreateZpoolRequestBuilder::default()
            .cache(file_path.clone())
            .build()
            .unwrap();

        let result: Vec<OsString> = topo.into_args();
        let expected = args_from_slice(&["cache", path]);

        assert_eq!(expected, result);

        // Zpool with mirror as ZIL and two vdevs
        let topo = CreateZpoolRequestBuilder::default()
            .vdev(naked_vdev.clone())
            .vdev(naked_vdev.clone())
            .zil(CreateVdevRequest::Mirror(get_disks(2, &file_path)))
            .build()
            .unwrap();

        let result = topo.into_args();
        let expected = args_from_slice(&[path, path, "log", "mirror", path, path]);
        assert_eq!(expected, result);

        // Zraid
        let topo = CreateZpoolRequestBuilder::default()
            .vdev(CreateVdevRequest::RaidZ(get_disks(3, &file_path)))
            .build()
            .unwrap();

        let result = topo.into_args();
        let expected = args_from_slice(&["raidz", path, path, path]);
        assert_eq!(expected, result);

        // Zraid 2
        let topo = CreateZpoolRequestBuilder::default()
            .vdev(CreateVdevRequest::RaidZ2(get_disks(5, &file_path)))
            .build()
            .unwrap();

        let result = topo.into_args();
        let expected = args_from_slice(&["raidz2", path, path, path, path, path]);
        assert_eq!(expected, result);

        // Zraid 3
        let topo = CreateZpoolRequestBuilder::default()
            .vdev(CreateVdevRequest::RaidZ3(get_disks(8, &file_path)))
            .build()
            .unwrap();

        let result = topo.into_args();
        let expected = args_from_slice(&["raidz3", path, path, path, path, path, path, path, path]);
        assert_eq!(expected, result);
    }
}
