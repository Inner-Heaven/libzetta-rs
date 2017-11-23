extern crate libzfs;
extern crate cavity;
extern crate tempdir;
extern crate slog_term;


use cavity::*;
use libzfs::slog::*;
use libzfs::zpool::{Disk, TopologyBuilder, Vdev, ZpoolEngine, ZpoolOpen3};
use std::fs::File;
use tempdir::TempDir;

fn get_logger() -> Logger {
    let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());
    Logger::root(slog_term::FullFormat::new(plain).build().fuse(), o!())
}

#[test]
fn create_check_delete() {
    let zpool = ZpoolOpen3::with_logger(get_logger());
    let tmp = TempDir::new("zpool-tests").unwrap();
    let file_path = tmp.path().join("device0");
    let mut file = File::create(file_path.clone()).unwrap();
    let name = "zpool-tests";

    fill(Bytes::MegaBytes(64), None, WriteMode::FlushOnce, &mut file).unwrap();

    let topo = TopologyBuilder::default()
        .vdev(Vdev::Naked(Disk::File(file_path.clone())))
        .build()
        .unwrap();

    zpool.create(&name, topo).unwrap();

    assert!(zpool.exists(&name).unwrap());

    zpool.destroy(&name, true).unwrap();

    assert!(!zpool.exists(&name).unwrap());
}
