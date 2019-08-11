/// CreateZpoolRequest is a structure that describes zpool vdev structure.
/// Use to create and updated zpool
use std::ffi::OsString;
use std::path::PathBuf;

use crate::zpool::{properties::ZpoolPropertiesWrite, vdev::CreateVdevRequest, CreateMode};

/// Structure representing what zpool consist of.
/// This structure is used in zpool creation and when new drives are attached.
///
/// ### Examples
///
/// Let's create simple topology: 2 drives in mirror, no l2arc, no zil.
///
/// ```rust
/// use libzfs::zpool::{CreateVdevRequest, CreateZpoolRequest};
/// use std::path::PathBuf;
///
/// let drives = vec![PathBuf::from("sd0"), PathBuf::from("sd1")];
/// let topo = CreateZpoolRequest::builder()
///     .name(String::from("tank"))
///     .vdevs(vec![CreateVdevRequest::Mirror(drives)])
///     .build()
///     .unwrap();
/// ```
/// Overkill example: 2 drives in mirror and a single drive, zil on double
/// mirror and 2 l2rc.
///
/// ```rust, norun
/// use libzfs::zpool::{CreateZpoolRequest, CreateVdevRequest};
/// use std::path::PathBuf;
///
/// let zil_drives = vec![PathBuf::from("hd0"), PathBuf::from("hd1")];
/// let mirror_drives = vec![PathBuf::from("hd2"), PathBuf::from("hd3")];
/// let cache_drives = vec![PathBuf::from("hd4"), PathBuf::from("hd5")];
/// let topo = CreateZpoolRequest::builder()
///     .name("tank")
///     .vdevs(vec![CreateVdevRequest::Mirror(mirror_drives)])
///     .cache("/tmp/sparse.file".into())
///     .vdev(CreateVdevRequest::SingleDisk(PathBuf::from("hd6")))
///     .caches(cache_drives)
///     .zil(CreateVdevRequest::Mirror(zil_drives))
///     .altroot(PathBuf::from("/mnt"))
///     .mount(PathBuf::from("/mnt"))
///     .build()
///     .unwrap();
/// ```

#[derive(Default, Builder, Debug, Clone, Getters, PartialEq, Eq)]
#[builder(setter(into))]
#[get = "pub"]
pub struct CreateZpoolRequest {
    /// Name to give new zpool
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
    /// cannot be mirrored. Since a cache device only stores additional copies
    /// of existing data, there is no risk of data loss.
    #[builder(default)]
    caches: Vec<PathBuf>,
    /// ZFS Log Devices, also known as ZFS Intent Log ([ZIL](https://www.freebsd.org/doc/handbook/zfs-term.html#zfs-term-zil)) move the intent log from the regular
    /// pool devices to a dedicated device, typically an SSD. Having a dedicated
    /// log device can significantly improve the performance of applications
    /// with a high volume of *synchronous* writes, especially databases.
    /// Log devices can be mirrored, but RAID-Z is not supported.
    /// If multiple log devices are used, writes will be load balanced across
    /// them
    #[builder(default)]
    logs: Vec<CreateVdevRequest>,
    /// The hot spares feature enables you to identify disks that could be used to replace a failed
    /// or faulted device in one or more storage pools. Designating a device as a hot spare means
    /// that the device is not an active device in the pool, but if an active device in the pool
    /// fails, the hot spare automatically replaces the failed device.
    #[builder(default)]
    spares: Vec<PathBuf>,
}

impl CreateZpoolRequest {
    /// Create builder
    pub fn builder() -> CreateZpoolRequestBuilder { CreateZpoolRequestBuilder::default() }

    /// Verify that given topology can be used to update existing pool.
    pub fn is_suitable_for_update(&self) -> bool {
        let valid_vdevs = self.vdevs.iter().all(CreateVdevRequest::is_valid);
        if !valid_vdevs {
            return false;
        }

        let valid_logs = self.logs.iter().all(CreateVdevRequest::is_valid);
        if !valid_logs {
            return false;
        }
        true
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
        ret.extend(vdevs);

        if !self.logs.is_empty() {
            let log_vdevs = self.logs.into_iter().flat_map(CreateVdevRequest::into_args);
            ret.push("log".into());
            ret.extend(log_vdevs);
        }

        if !self.caches.is_empty() {
            let caches = self.caches.into_iter().map(PathBuf::into_os_string);
            ret.push("cache".into());
            ret.extend(caches);
        }

        if !self.spares.is_empty() {
            let spares = self.spares.into_iter().map(PathBuf::into_os_string);
            ret.push("spare".into());
            ret.extend(spares);
        }
        ret
    }
}

impl CreateZpoolRequestBuilder {
    /// Add vdev to request.
    ///
    /// * `vdev` - [CreateVdevRequest](struct.CreateVdevRequest.html) for vdev.
    pub fn vdev(&mut self, vdev: CreateVdevRequest) -> &mut CreateZpoolRequestBuilder {
        match self.vdevs {
            Some(ref mut vec) => vec.push(vdev),
            None => {
                self.vdevs = Some(Vec::new());
                return self.vdev(vdev);
            },
        }
        self
    }

    /// Add cache device to request.
    ///
    /// * `disk` - path to file or name of block device in `/dev/`. Some ZFS implementations forbid
    ///   using files as cache.
    pub fn cache(&mut self, disk: PathBuf) -> &mut CreateZpoolRequestBuilder {
        match self.caches {
            Some(ref mut vec) => vec.push(disk),
            None => {
                self.caches = Some(Vec::new());
                return self.cache(disk);
            },
        }
        self
    }

    /// Add Vdev that will be used as ZFS Intent Log to request.
    ///
    /// * `vdev` - [CreateVdevRequest](struct.CreateVdevRequest.html) for ZIL device.
    pub fn zil(&mut self, log: CreateVdevRequest) -> &mut CreateZpoolRequestBuilder {
        match self.logs {
            Some(ref mut vec) => vec.push(log),
            None => {
                self.logs = Some(Vec::with_capacity(1));
                return self.zil(log);
            },
        }
        self
    }

    /// Add spare disk that will be used to replace failed device in zpool.
    ///
    /// * `disk` - path to file or name of block device in `/dev/`.
    pub fn spare(&mut self, disk: PathBuf) -> &mut CreateZpoolRequestBuilder {
        match self.spares {
            Some(ref mut vec) => vec.push(disk),
            None => {
                self.spares = Some(Vec::new());
                return self.spare(disk);
            },
        }
        self
    }
}

#[cfg(test)]
mod test {
    use std::{fs::File, path::PathBuf};

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
            .name("tank")
            .vdevs(vec![CreateVdevRequest::Mirror(get_disks(2, &file_path))])
            .build()
            .unwrap();

        assert!(topo.is_suitable_for_create());

        // Zpool with invalid mirror
        let topo = CreateZpoolRequestBuilder::default()
            .name("tank")
            .vdevs(vec![CreateVdevRequest::Mirror(get_disks(1, &file_path))])
            .build()
            .unwrap();

        assert!(!topo.is_suitable_for_create());

        // Zpool with valid cache and valid vdev
        let topo = CreateZpoolRequestBuilder::default()
            .name("tank")
            .vdevs(vec![CreateVdevRequest::Mirror(get_disks(2, &file_path))])
            .caches(get_disks(2, &file_path))
            .build()
            .unwrap();

        assert!(topo.is_suitable_for_create());

        // Just add L2ARC to zpool
        let topo =
            CreateZpoolRequestBuilder::default().name("tank").cache(file_path).build().unwrap();

        assert!(topo.is_suitable_for_update());
        assert!(!topo.is_suitable_for_create());
    }

    #[test]
    fn test_builder() {
        let result = CreateZpoolRequest::builder().build();
        assert!(result.is_err());
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
            .name("tank")
            .cache(file_path.clone())
            .build()
            .unwrap();

        let result: Vec<OsString> = topo.into_args();
        let expected = args_from_slice(&["cache", path]);

        assert_eq!(expected, result);

        // Zpool with mirror as ZIL and two vdevs
        let topo = CreateZpoolRequestBuilder::default()
            .name("tank")
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
            .name("tank")
            .vdev(CreateVdevRequest::RaidZ(get_disks(3, &file_path)))
            .build()
            .unwrap();

        let result = topo.into_args();
        let expected = args_from_slice(&["raidz", path, path, path]);
        assert_eq!(expected, result);

        // Zraid 2
        let topo = CreateZpoolRequestBuilder::default()
            .name("tank")
            .vdev(CreateVdevRequest::RaidZ2(get_disks(5, &file_path)))
            .build()
            .unwrap();

        let result = topo.into_args();
        let expected = args_from_slice(&["raidz2", path, path, path, path, path]);
        assert_eq!(expected, result);

        // Zraid 3
        let topo = CreateZpoolRequestBuilder::default()
            .name("tank")
            .vdev(CreateVdevRequest::RaidZ3(get_disks(8, &file_path)))
            .build()
            .unwrap();

        let result = topo.into_args();
        let expected = args_from_slice(&["raidz3", path, path, path, path, path, path, path, path]);
        assert_eq!(expected, result);
    }
}
