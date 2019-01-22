extern crate cavity;
#[macro_use]
extern crate lazy_static;
extern crate libzfs;
extern crate rand;
extern crate slog_term;
extern crate tempdir;

use std::fs;
use std::fs::DirBuilder;
use std::panic;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use cavity::{Bytes, fill, WriteMode};
use rand::Rng;

use libzfs::slog::*;
use libzfs::zpool::{CreateVdevRequest, CreateZpoolRequestBuilder, Disk, ZpoolEngine, ZpoolOpen3};
use libzfs::zpool::{FailMode, ZpoolError, ZpoolErrorKind, ZpoolPropertiesWriteBuilder};
use libzfs::zpool::{CreateMode, Health, OfflineMode, OnlineMode};

static ZPOOL_NAME_PREFIX: &'static str = "tests";
lazy_static! {
    static ref SHARED: Mutex<u8> = Mutex::new(0);
}
fn get_zpool_name() -> String {
    let mut rng = rand::thread_rng();
    let suffix = rng.gen::<u64>();
    let name = format!("{}-{}", ZPOOL_NAME_PREFIX, suffix);
    name
}
fn setup_vdev<P: AsRef<Path>>(path: P, bytes: &Bytes) -> PathBuf {
    let path = path.as_ref();

    let parent = path.parent().unwrap();
    DirBuilder::new().recursive(true).create(parent).unwrap();

    if path.exists() {
        let meta = fs::metadata(&path).unwrap();
        assert!(meta.is_file());
        assert!(!meta.permissions().readonly());
        if (meta.len() as usize) < bytes.as_bytes() {
            let _ = fs::remove_file(&path);
            setup_vdev(path, bytes)
        } else {
            path.into()
        }
    } else {
        let mut f = fs::File::create(path).unwrap();
        fill(bytes.clone(), None, WriteMode::FlushOnce, &mut f).unwrap();
        path.into()
    }
}
fn setup() {
    // Create vdevs if they're missing
    let vdev_dir = Path::new("/vdevs");
    setup_vdev(vdev_dir.join("vdev0"), &Bytes::MegaBytes(64 + 10));
    setup_vdev(vdev_dir.join("vdev1"), &Bytes::MegaBytes(64 + 10));
    setup_vdev(vdev_dir.join("vdev2"), &Bytes::MegaBytes(64 + 10));
    setup_vdev(vdev_dir.join("vdev3"), &Bytes::MegaBytes(1));
}
fn run_test<T>(test: T)
    where
    T: FnOnce(String) -> () + panic::UnwindSafe,
{
    let lock = SHARED.lock().unwrap();
    setup();

    let name = get_zpool_name();
    let result = panic::catch_unwind(|| {
        test(name.clone());
    });

    let zpool = ZpoolOpen3::default();
    let _ = zpool.destroy(&name, true);
    drop(lock);

    result.unwrap();
}

// Only used for debugging
#[allow(dead_code)]
fn get_logger() -> Logger {
    let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());
    Logger::root(
        slog_term::FullFormat::new(plain)
            .use_original_order()
            .build()
            .fuse(),
        o!(),
    )
}

#[test]
fn create_check_update_delete() {
    run_test(|name| {
        let zpool = ZpoolOpen3::default();

        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .vdev(CreateVdevRequest::SingleDisk("/vdevs/vdev0".into()))
            .build()
            .unwrap();

        zpool.create(topo).unwrap();

        let result = zpool.exists(&name).unwrap();
        assert!(result);

        let props = zpool.read_properties(&name).unwrap();
        let updated_props = ZpoolPropertiesWriteBuilder::from_props(&props)
            .auto_expand(true)
            .auto_replace(true)
            .comment("Wat")
            .fail_mode(FailMode::Panic)
            .build()
            .unwrap();

        zpool.update_properties(&name, updated_props).unwrap();
        let props = zpool.read_properties(&name).unwrap();
        assert_eq!(true, props.auto_expand);
        assert_eq!(true, props.auto_replace);
        assert_eq!(Some(String::from("Wat")), props.comment);
        assert_eq!(FailMode::Panic, props.fail_mode);

        let updated_props = ZpoolPropertiesWriteBuilder::from_props(&props)
            .comment("Wat")
            .build()
            .unwrap();
        zpool.update_properties(&name, updated_props).unwrap();
        let props = zpool.read_properties(&name).unwrap();
        assert_eq!(true, props.auto_expand);
        assert_eq!(true, props.auto_replace);
        assert_eq!(Some(String::from("Wat")), props.comment);
        assert_eq!(FailMode::Panic, props.fail_mode);

        let updated_props = ZpoolPropertiesWriteBuilder::from_props(&props)
            .comment(String::new())
            .delegation(true)
            .build()
            .unwrap();
        zpool.update_properties(&name, updated_props).unwrap();
        let props = zpool.read_properties(&name).unwrap();
        assert_eq!(None, props.comment);
        assert_eq!(true, props.delegation);

        zpool.destroy(&name, true).unwrap();

        let result = zpool.exists(&name).unwrap();
        assert!(!result);
    })
}

#[test]
fn cmd_not_found() {
    run_test(|name| {
        let zpool = ZpoolOpen3::with_cmd("zpool-not-found");

        let topo = CreateZpoolRequestBuilder::default()
            .name(name)
            .vdev(CreateVdevRequest::SingleDisk("/vdevs/vdev0".into()))
            .build()
            .unwrap();

        let result = zpool.create(topo);
        assert_eq!(ZpoolErrorKind::CmdNotFound, result.unwrap_err().kind());

        let result = zpool.exists("wat");
        assert_eq!(ZpoolErrorKind::CmdNotFound, result.unwrap_err().kind());
    });
}

#[test]
fn reuse_vdev() {
    run_test(|name_1| {
        let zpool = ZpoolOpen3::default();
        let name_2 = "zpool-tests-fail";
        let vdev_file = "/vdevs/vdev1";

        let props = ZpoolPropertiesWriteBuilder::default().build().unwrap();
        let topo1 = CreateZpoolRequestBuilder::default()
            .name(name_1.clone())
            .props(props.clone())
            .vdev(CreateVdevRequest::SingleDisk(vdev_file.into()))
            .build()
            .unwrap();
        let topo2 = CreateZpoolRequestBuilder::default()
            .name(name_2.clone())
            .vdev(CreateVdevRequest::SingleDisk(vdev_file.into()))
            .build()
            .unwrap();


        let result = zpool.create(topo1);
        result.unwrap();
        let result = zpool.create(topo2);
        let err = result.unwrap_err();
        assert_eq!(ZpoolErrorKind::VdevReuse, err.kind());
        println!("{:?}", &err);
        if let ZpoolError::VdevReuse(vdev, pool) = err {
            assert_eq!(vdev_file, vdev);
            assert_eq!(name_1, pool);
        }
        zpool.destroy(&name_1, true).unwrap();
    });
}
#[test]
fn create_invalid_topo() {
    let zpool = ZpoolOpen3::default();
    let name = get_zpool_name();

    let topo = CreateZpoolRequestBuilder::default()
        .name(name)
        .cache(PathBuf::from("/vdevs/vdev0"))
        .build()
        .unwrap();

    let result = zpool.create(topo);

    let err = result.unwrap_err();
    assert_eq!(ZpoolErrorKind::InvalidTopology, err.kind());
}

#[test]
fn pool_not_found() {
    let zpool = ZpoolOpen3::default();
    let name = get_zpool_name();

    let err = zpool.destroy(&name, true).unwrap_err();

    assert_eq!(ZpoolErrorKind::PoolNotFound, err.kind());

    let err = zpool.read_properties(&name).unwrap_err();
    assert_eq!(ZpoolErrorKind::PoolNotFound, err.kind());

    let props = ZpoolPropertiesWriteBuilder::default().build().unwrap();
    let err = zpool.update_properties(&name, props).unwrap_err();
    assert_eq!(ZpoolErrorKind::PoolNotFound, err.kind());

    let err = zpool.export("fake", true).unwrap_err();
    assert_eq!(ZpoolErrorKind::PoolNotFound, err.kind());
}

#[test]
fn read_args() {
    run_test(|name| {
        let zpool = ZpoolOpen3::default();

        let vdev_path = setup_vdev("/vdevs/vdev0", &Bytes::MegaBytes(64 + 10));
        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .vdev(CreateVdevRequest::disk(vdev_path))
            .build()
            .unwrap();

        zpool.create(topo).unwrap();

        let props = zpool.read_properties(&name);

        assert!(props.is_ok());
        zpool.destroy(&name, true).unwrap();
    });
}

#[test]
fn create_mount() {
    run_test(|name| {
        let zpool = ZpoolOpen3::default();
        let mut mount_point = PathBuf::from("/tmp");
        mount_point.push(&name);

        let vdev_path = setup_vdev("/vdevs/vdev0", &Bytes::MegaBytes(64 + 10));
        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .mount(mount_point.clone())
            .vdev(CreateVdevRequest::disk(vdev_path))
            .build()
            .unwrap();

        assert!(!mount_point.exists());
        let result = zpool.create(topo);
        result.unwrap();
        assert!(mount_point.exists());
        zpool.destroy(&name, true).unwrap();
    });
}

#[test]
fn create_mount_and_alt_root() {
    run_test(|name| {
        let zpool = ZpoolOpen3::default();
        let mut mount_point = PathBuf::from("/tmp");
        mount_point.push(&name);

        let mut expected = PathBuf::from("/mnt/tmp");
        expected.push(&name);

        let alt_root = PathBuf::from("/mnt");

        let vdev_path = setup_vdev("/vdevs/vdev0", &Bytes::MegaBytes(64 + 10));
        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .mount(mount_point.clone())
            .altroot(alt_root.clone())
            .vdev(CreateVdevRequest::disk(vdev_path))
            .build()
            .unwrap();

        let result = zpool.create(topo);
        result.unwrap();

        let props = zpool.read_properties(&name).unwrap();
        assert_eq!(props.alt_root, Some(PathBuf::from("/mnt")));

        assert!(expected.exists());
        zpool.destroy(&name, true).unwrap();
    });
}
#[test]
fn create_with_props() {
    run_test(|name| {
        let zpool = ZpoolOpen3::default();
        let comment = String::from("this is a comment");

        let alt_root = PathBuf::from("/mnt");
        let vdev_path = setup_vdev("/vdevs/vdev0", &Bytes::MegaBytes(64 + 10));
        let props = ZpoolPropertiesWriteBuilder::default()
            .auto_expand(true)
            .comment(comment.clone())
            .fail_mode(FailMode::Panic)
            .build()
            .unwrap();

        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .mount(alt_root.clone())
            .altroot(alt_root.clone())
            .vdev(CreateVdevRequest::disk(vdev_path))
            .props(props.clone())
            .build()
            .unwrap();


        zpool.create(topo).unwrap();

        let props = zpool.read_properties(&name).unwrap();
        assert_eq!(true, props.auto_expand);
        assert_eq!(FailMode::Panic, props.fail_mode);
        assert_eq!(Some(comment.clone()), props.comment);
        zpool.destroy(&name, true).unwrap();
    });
}

#[test]
fn test_export_import() {
    run_test(|name| {
        let vdev_dir = Path::new("/vdevs/import");
        setup_vdev(vdev_dir.join("vdev0"), &Bytes::MegaBytes(64 + 10));
        let zpool = ZpoolOpen3::default();
        //let zpool = ZpoolOpen3::with_logger(get_logger());

        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .vdev(CreateVdevRequest::SingleDisk("/vdevs/import/vdev0".into()))
            .build()
            .unwrap();
        zpool.create(topo).expect("Failed to create pool for export");

        let result = zpool.export(&name, false);
        assert!(result.is_ok());
        let list = zpool.available_in_dir(PathBuf::from(&vdev_dir)).unwrap();
        assert_eq!(list.len(), 1);

        let result = zpool.import_from_dir(&name, PathBuf::from(vdev_dir));
        assert!(result.is_ok());

        zpool.destroy(&name, true).unwrap();

        let result = zpool.available().unwrap();
        assert!(result.is_empty());
    });
}

#[test]
fn test_status() {
    run_test(|name| {
        let vdev_path = setup_vdev("/vdevs/vdev0", &Bytes::MegaBytes(64 + 10));
        let zpool = ZpoolOpen3::default();

        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .vdev(CreateVdevRequest::SingleDisk(vdev_path))
            .build()
            .unwrap();
        zpool.create(topo.clone()).unwrap();


        let result = zpool.status(&name).unwrap();
        assert_eq!(&name, result.name());
        assert_eq!(&topo, result.topology());
    });
}
#[test]
fn test_all() {
    run_test(|name| {
        let zpool = ZpoolOpen3::default();
        let vdev_path = setup_vdev("/vdevs/vdev0", &Bytes::MegaBytes(64 + 10));
        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .vdev(CreateVdevRequest::SingleDisk(vdev_path))
            .build()
            .unwrap();
        zpool.create(topo.clone()).unwrap();


        let result = zpool.all().unwrap();
        assert_eq!(1, result.len());
        let result = result.into_iter().next().unwrap();
        assert_eq!(&name, result.name());
        assert_eq!(&topo, result.topology());
    });
}

#[test]
fn test_zpool_with_logger() {
    let _zpool = ZpoolOpen3::with_logger(get_logger());
}

#[test]
fn test_zpool_scrub_not_found() {
    let zpool = ZpoolOpen3::default();
    let name = "non-existent";

    let result = zpool.scrub(name);
    assert_eq!(ZpoolErrorKind::PoolNotFound, result.unwrap_err().kind());

    let result = zpool.stop_scrub(name);
    assert_eq!(ZpoolErrorKind::PoolNotFound, result.unwrap_err().kind());
}

#[test]
fn test_zpool_scrub() {
    run_test(|name| {
        let zpool = ZpoolOpen3::default();
        let vdev_path = setup_vdev("/vdevs/vdev0", &Bytes::MegaBytes(64 + 10));
        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .vdev(CreateVdevRequest::SingleDisk(vdev_path))
            .build()
            .unwrap();
        zpool.create(topo).unwrap();

        let result = zpool.stop_scrub(&name);
        assert_eq!(ZpoolErrorKind::NoActiveScrubs, result.unwrap_err().kind());

        let result = zpool.pause_scrub(&name);
        assert_eq!(ZpoolErrorKind::NoActiveScrubs, result.unwrap_err().kind());

        let result = zpool.scrub(&name);
        assert!(result.is_ok());
    });
}

#[test]
fn test_zpool_take_single_device_offline() {
    run_test(|name| {
        let zpool = ZpoolOpen3::default();
        let vdev_path = setup_vdev("/vdevs/vdev0", &Bytes::MegaBytes(64 + 10));
        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .vdev(CreateVdevRequest::SingleDisk(vdev_path.clone()))
            .build()
            .unwrap();
        zpool.create(topo).unwrap();

        let disk0 = Disk::File(vdev_path.clone());
        let result = zpool.take_offline(&name, &disk0, OfflineMode::UntilReboot);
        dbg!(&result);
        assert!(result.is_err());

        assert_eq!(result.unwrap_err().kind(), ZpoolErrorKind::NoValidReplicas);
    });
}

#[test]
fn test_zpool_take_device_from_mirror_offline() {
    run_test(|name| {
        let zpool = ZpoolOpen3::default();
        let vdev0_path = setup_vdev("/vdevs/vdev3", &Bytes::MegaBytes(64 + 10));
        let vdev1_path = setup_vdev("/vdevs/vdev4", &Bytes::MegaBytes(64 + 10));
        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::Mirror(vec![
                vdev0_path.clone(),
                vdev1_path.clone()
            ]))
            .build()
            .unwrap();
        zpool.create(topo).unwrap();
        let disk0 = Disk::File(vdev0_path.clone());
        let result = zpool.take_offline(&name, &disk0, OfflineMode::UntilReboot);
        assert!(result.is_ok());

        let z = zpool.status(&name).unwrap();
        assert_eq!(&Health::Degraded, z.health());

        let result = zpool.bring_online(&name, &disk0, OnlineMode::Simple);
        assert!(result.is_ok());

        let z = zpool.status(&name).unwrap();
        assert_eq!(&Health::Online, z.health());
    });
}