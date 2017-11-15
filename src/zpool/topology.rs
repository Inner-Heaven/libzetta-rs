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
