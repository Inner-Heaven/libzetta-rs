extern crate libzfs;
extern crate tempdir;
extern crate slog_term;

use libzfs::slog::*;
use libzfs::zpool::{Disk, TopologyBuilder, Vdev, ZpoolEngine, ZpoolOpen3};
use libzfs::zpool::{ZpoolError, ZpoolErrorKind};

fn get_logger() -> Logger {
    let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());
    Logger::root(slog_term::FullFormat::new(plain).build().fuse(), o!())
}

#[test]
fn create_check_delete() {
    let zpool = ZpoolOpen3::with_logger(get_logger());
    let name = "zpool-tests";


    let topo = TopologyBuilder::default()
        .vdev(Vdev::Naked(Disk::File("/vdevs/vdev0".into())))
        .build()
        .unwrap();

    zpool.create(&name, topo).unwrap();

    assert!(zpool.exists(&name).unwrap());

    zpool.destroy(&name, true).unwrap();

    assert!(!zpool.exists(&name).unwrap());
}


#[test]
fn cmd_not_found() {
    let zpool = ZpoolOpen3::with_cmd("zpool-not-found");
    let name = "zpool-tests";

    let topo = TopologyBuilder::default()
        .vdev(Vdev::Naked(Disk::File("/vdevs/vdev0".into())))
        .build()
        .unwrap();

    let result = zpool.create(&name, topo);
    assert!(result.is_err());
    assert_eq!(ZpoolErrorKind::CmdNotFound, result.unwrap_err().kind());
}

#[test]
fn reuse_vdev() {
    let zpool = ZpoolOpen3::default();
    let name_1 = "zpool-tests";
    let name_2 = "zpool-tests-fail";
    let vdev_file = "/vdevs/vdev0";

    let topo = TopologyBuilder::default()
        .vdev(Vdev::Naked(Disk::File(vdev_file.into())))
        .build()
        .unwrap();

    let result = zpool.create(&name_1, topo.clone());
    assert!(result.is_ok());
    let result = zpool.create(&name_2, topo.clone());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(ZpoolErrorKind::VdevReuse, err.kind());
    println!("{:?}", &err);
    if let ZpoolError::VdevReuse(vdev, pool) = err {
        assert_eq!(vdev_file, vdev);
        assert_eq!(name_1, pool);
    }
    zpool.destroy(&name_1, true).unwrap();
}
