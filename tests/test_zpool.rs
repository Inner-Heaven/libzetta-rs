extern crate libzfs;
extern crate cavity;
extern crate tempdir;

use libzfs::zpool::{ZpoolEngine, ZpoolOpen3, Vdev, TopologyBuilder, Disk};
use cavity::*;
use tempdir::TempDir;
use std::fs::File;

#[test]
fn create_check_delete() {
    let zpool = ZpoolOpen3::default();
    let tmp = TempDir::new("zpool-tests").unwrap();
    let file_path = tmp.path().join("device0");
    let mut file = File::create(file_path.clone()).unwrap();
    let name = "zpool-tests";

    fill(Bytes::MegaBytes(64), None, WriteMode::FlushOnce, &mut file).unwrap();

    let topo = TopologyBuilder::default()
        .vdev(Vdev::Naked(Disk::File(file_path.clone())))
        .build()
        .unwrap();

    zpool.create(&name, topo, None).unwrap();

    assert!(zpool.exists(&name).unwrap());

    zpool.destroy(&name, true).unwrap();

    assert!(!zpool.exists(&name).unwrap());
}
