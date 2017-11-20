/// Topology is a structure that describes zpool vdev structure.
/// Use to create and updated zpool

use std::ffi::OsString;
use zpool::vdev::{Vdev, Disk};

/// Structure representing what zpool consist of.
/// This structure is used in zpool creation and when new drives are attached.
///
/// ### Examples
///
/// Let's create simple topology: 2 drives in mirror, no l2arc, no zil.
///
/// ```rust
/// use libzfs::zpool::{Disk,Vdev};
/// use libzfs::zpool::{TopologyBuilder};
///
/// let drives = vec![Disk::disk("hd0"), Disk::disk("hd1")];
/// let topo = TopologyBuilder::default()
///             .vdevs(vec![Vdev::Mirror(drives)])
///             .build()
///             .unwrap();
/// ```
/// Overkill example: 2 drives in mirror and a single drive, zil on double mirror and 2 l2rc.
///
/// ```rust
/// use libzfs::zpool::{Disk,Vdev};
/// use libzfs::zpool::{TopologyBuilder};
///
///
/// let zil_drives = vec![Disk::Disk("hd0".into()), Disk::Disk("hd1".into())];
/// let mirror_drives = vec![Disk::Disk("hd2".into()), Disk::Disk("hd3".into())];
/// let cache_drives = vec![Disk::Disk("hd4".into()), Disk::Disk("hd5".into())];
/// let topo = TopologyBuilder::default()
///             .vdevs(vec![Vdev::Mirror(mirror_drives)])
///             .cache(Disk::File("/tmp/sparse.file".into()))
///             .vdev(Vdev::Naked(Disk::Disk("sd0".into())))
///             .caches(cache_drives)
///             .zil(Vdev::Mirror(zil_drives))
///             .build()
///             .unwrap();
/// ```

#[allow(unused_mut)]
#[derive(Default, Builder, Debug)]
#[builder(setter(into))]
pub struct Topology {
    #[builder(default)]
    vdevs: Vec<Vdev>,
    #[builder(default)]
    caches: Vec<Disk>,
    #[builder(default)]
    zil: Option<Vdev>

}

impl Topology {
    /// Verify that given topology can be used to update existing pool.
    pub fn is_suitable_for_update(&self) -> bool {
        let valid_vdevs = self.vdevs.iter().all(Vdev::is_valid);
        if !valid_vdevs {
            return false;
        }

        if !self.caches.is_empty() {
            let valid_caches = self.caches.iter().all(Disk::is_valid);
            if !valid_caches {
                return false;
            }
        }

        match self.zil {
            Some(ref vdev) => return vdev.is_valid(),
            None => return true
        }
    }
    /// Verify that given topology can be used to create new zpool.
    ///
    /// That means it as at least one valid vdev and all optional devices are valid if present.
    pub fn suitable_for_create(&self) -> bool {
        if self.vdevs.len() < 1 {
            return false;
        }
        self.is_suitable_for_update()
    }

    /// Make Topology usable as arg for Command
    pub fn into_args(self) -> Vec<OsString> {
        let mut ret: Vec<OsString> = Vec::with_capacity(13);

        let vdevs = self.vdevs.into_iter().flat_map(Vdev::into_args);
        let zil = self.zil.map(Vdev::into_args);
        ret.extend(vdevs);

        if !self.caches.is_empty() {
            let caches = self.caches.into_iter().map(Disk::into_arg);
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

impl TopologyBuilder {
    pub fn vdev(&mut self, vdev: Vdev) -> &mut TopologyBuilder {
        match self.vdevs {
            Some(ref mut vec) => vec.push(vdev),
            None => {
                self.vdevs = Some(Vec::new());
                return self.vdev(vdev);
            }
        }
        self
    }

    pub fn cache(&mut self, disk: Disk) -> &mut TopologyBuilder {
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
    use super::*;

    use std::fs::File;
    use std::path::PathBuf;
    use tempdir::TempDir;

    fn get_disks(num: usize, path: &PathBuf) -> Vec<Disk> {
        (0..num).map(|_| Disk::File(path.clone())).collect()
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
        let topo = TopologyBuilder::default()
            .vdevs(vec![Vdev::Mirror(get_disks(2, &file_path))])
            .build()
            .unwrap();

        assert!(topo.suitable_for_create());

        // Zpool with invalid mirror
        let topo = TopologyBuilder::default()
            .vdevs(vec![Vdev::Mirror(get_disks(1, &file_path))])
            .build()
            .unwrap();

        assert!(!topo.suitable_for_create());

        // Zpool with valid cache and valid vdev
        let topo = TopologyBuilder::default()
            .vdevs(vec![Vdev::Mirror(get_disks(2, &file_path))])
            .caches(get_disks(2, &file_path))
            .build()
            .unwrap();

        assert!(topo.suitable_for_create());

        // Zpool with valid mirror, but invalid cache
        let invalid_path = tmp_dir.path().join("fake");
        let topo = TopologyBuilder::default()
            .vdevs(vec![Vdev::Mirror(get_disks(2, &file_path))])
            .caches(get_disks(2, &file_path))
            .cache(Disk::File(invalid_path.clone()))
            .build()
            .unwrap();

        assert!(!topo.suitable_for_create());

        // Zpool with invalid zil
        let invalid_path = tmp_dir.path().join("fake");
        let topo = TopologyBuilder::default()
            .vdevs(vec![Vdev::Mirror(get_disks(2, &file_path))])
            .caches(get_disks(2, &file_path))
            .zil(Vdev::Naked(Disk::File(invalid_path)))
            .build()
            .unwrap();

        assert!(!topo.suitable_for_create());

        // Zpool with invalid zil
        let invalid_path = tmp_dir.path().join("fake");
        let topo = TopologyBuilder::default()
            .vdevs(vec![Vdev::Mirror(get_disks(2, &file_path))])
            .caches(get_disks(2, &file_path))
            .zil(Vdev::Naked(Disk::File(invalid_path.clone())))
            .build()
            .unwrap();

        assert!(!topo.suitable_for_create());


        // Just add L2ARC to zpool
        let topo = TopologyBuilder::default()
            .cache(Disk::File(file_path.clone()))
            .build()
            .unwrap();

        assert!(topo.is_suitable_for_update());

        // Add L2ARC and invalid vdev
        let invalid_path = tmp_dir.path().join("fake");
        let topo = TopologyBuilder::default()
            .cache(Disk::File(file_path.clone()))
            .vdev(Vdev::Naked(Disk::File(invalid_path)))
            .vdev(Vdev::Naked(Disk::File(file_path.clone())))
            .build()
            .unwrap();

        assert!(!topo.is_suitable_for_update());
    }

    #[test]
    fn test_args() {
        let tmp_dir = TempDir::new("zpool-tests").unwrap();
        let file_path = tmp_dir.path().join("block-device");
        let path = file_path.to_str().unwrap();
        let _valid_file = File::create(file_path.clone()).unwrap();
        let naked_vdev = Vdev::Naked(Disk::File(file_path.clone()));

        // Just add L2ARC to zpool
        let topo = TopologyBuilder::default()
            .cache(Disk::File(file_path.clone()))
            .build()
            .unwrap();

        let result: Vec<OsString> = topo.into_args();
        let expected = args_from_slice(&["cache", path]);

        assert_eq!(expected, result);


        // Zpool with mirror as ZIL and two vdevs
        let topo = TopologyBuilder::default()
            .vdev(naked_vdev.clone())
            .vdev(naked_vdev.clone())
            .zil(Vdev::Mirror(get_disks(2, &file_path)))
            .build()
            .unwrap();

        let result = topo.into_args();
        let expected = args_from_slice(&[path, path, "log", "mirror", path, path]);
        assert_eq!(expected, result);


        // Zraid
        let topo = TopologyBuilder::default()
            .vdev(Vdev::RaidZ(get_disks(3, &file_path)))
            .build()
            .unwrap();

        let result = topo.into_args();
        let expected = args_from_slice(&["raidz", path, path, path]);
        assert_eq!(expected, result);

        // Zraid 2
        let topo = TopologyBuilder::default()
            .vdev(Vdev::RaidZ2(get_disks(5, &file_path)))
            .build()
            .unwrap();

        let result = topo.into_args();
        let expected = args_from_slice(&["raidz2", path, path, path, path, path]);
        assert_eq!(expected, result);

        // Zraid 3
        let topo = TopologyBuilder::default()
            .vdev(Vdev::RaidZ3(get_disks(8, &file_path)))
            .build()
            .unwrap();

        let result = topo.into_args();
        let expected = args_from_slice(&["raidz3", path, path, path, path, path, path, path, path]);
        assert_eq!(expected, result);
    }
}
