/// Topology is a structure that describes zpool vdev structure.
/// Use to create and updated zpool

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
    /// Verify that given topology can be used to create new zpool.
    ///
    /// That means it as at least one valid vdev and all optional devices are valid if present.
    pub fn suitable_for_create(&self) -> bool {
        if self.vdevs.len() < 1 {
            return false;
        }

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

    #[test]
    fn test_create() {
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
        let caches = get_disks(1, &file_path);
        let topo = TopologyBuilder::default()
            .vdevs(vec![Vdev::Mirror(get_disks(2, &file_path))])
            .caches(get_disks(2, &file_path))
            .zil(Vdev::Naked(Disk::File(file_path.clone())))
            .build()
            .unwrap();

        assert!(topo.suitable_for_create());

        // Zpool with invalid zil
        let invalid_path = tmp_dir.path().join("fake");
        let caches = get_disks(1, &file_path);
        let topo = TopologyBuilder::default()
            .vdevs(vec![Vdev::Mirror(get_disks(2, &file_path))])
            .caches(get_disks(2, &file_path))
            .zil(Vdev::Naked(Disk::File(invalid_path.clone())))
            .build()
            .unwrap();

        assert!(!topo.suitable_for_create());
    }
}
