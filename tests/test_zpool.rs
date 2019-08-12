#[macro_use] extern crate lazy_static;

use rand;
use slog_term;

use std::{fs::{self, DirBuilder},
          panic,
          path::{Path, PathBuf},
          sync::Mutex};

use cavity::{fill, Bytes, WriteMode};
use rand::Rng;

use libzetta::{slog::*,
             zpool::{CreateMode, CreateVdevRequest, CreateZpoolRequestBuilder, DestroyMode,
                     ExportMode, FailMode, Health, OfflineMode, OnlineMode, ZpoolEngine,
                     ZpoolError, ZpoolErrorKind, ZpoolOpen3, ZpoolPropertiesWriteBuilder}};

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
    let _ = zpool.destroy(&name, DestroyMode::Force);
    drop(lock);

    result.unwrap();
}

#[cfg(target_os = "freebsd")]
fn get_virtual_device() -> PathBuf { PathBuf::from("md1") }

#[cfg(target_os = "linux")]
fn get_virtual_device() -> PathBuf { PathBuf::from("loop0") }

// Only used for debugging
#[allow(dead_code)]
fn get_logger() -> Logger {
    let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());
    Logger::root(slog_term::FullFormat::new(plain).use_original_order().build().fuse(), o!())
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
        assert_eq!(&true, props.auto_expand());
        assert_eq!(&true, props.auto_replace());
        assert_eq!(&Some(String::from("Wat")), props.comment());
        assert_eq!(&FailMode::Panic, props.fail_mode());

        let updated_props =
            ZpoolPropertiesWriteBuilder::from_props(&props).comment("Wat").build().unwrap();
        zpool.update_properties(&name, updated_props).unwrap();
        let props = zpool.read_properties(&name).unwrap();
        assert_eq!(&true, props.auto_expand());
        assert_eq!(&true, props.auto_replace());
        assert_eq!(&Some(String::from("Wat")), props.comment());
        assert_eq!(&FailMode::Panic, props.fail_mode());

        let updated_props = ZpoolPropertiesWriteBuilder::from_props(&props)
            .comment(String::new())
            .delegation(true)
            .build()
            .unwrap();
        zpool.update_properties(&name, updated_props).unwrap();
        let props = zpool.read_properties(&name).unwrap();
        assert_eq!(&None, props.comment());
        assert_eq!(&true, props.delegation());

        zpool.destroy(&name, DestroyMode::Force).unwrap();

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
        zpool.destroy(&name_1, DestroyMode::Force).unwrap();
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

    let err = zpool.read_properties(&name).unwrap_err();
    assert_eq!(ZpoolErrorKind::PoolNotFound, err.kind());

    let props = ZpoolPropertiesWriteBuilder::default().build().unwrap();
    let err = zpool.update_properties(&name, props).unwrap_err();
    assert_eq!(ZpoolErrorKind::PoolNotFound, err.kind());

    let err = zpool.export("fake", ExportMode::Gentle).unwrap_err();
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
        zpool.destroy(&name, DestroyMode::Force).unwrap();
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
        zpool.destroy(&name, DestroyMode::Force).unwrap();
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
        assert_eq!(&Some(PathBuf::from("/mnt")), props.alt_root());

        assert!(expected.exists());
        zpool.destroy(&name, DestroyMode::Force).unwrap();
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
        assert_eq!(&true, props.auto_expand());
        assert_eq!(&FailMode::Panic, props.fail_mode());
        assert_eq!(&Some(comment.clone()), props.comment());
        zpool.destroy(&name, DestroyMode::Force).unwrap();
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

        let result = zpool.export(&name, ExportMode::Gentle);
        assert!(result.is_ok());
        let list = zpool.available_in_dir(PathBuf::from(&vdev_dir)).unwrap();
        assert_eq!(list.len(), 1);

        let result = zpool.import_from_dir(&name, PathBuf::from(vdev_dir));
        assert!(result.is_ok());

        zpool.destroy(&name, DestroyMode::Force).unwrap();

        let result = zpool.available().unwrap();
        assert!(result.is_empty());
    });
}

#[test]
fn test_export_import_force() {
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

        let result = zpool.export(&name, ExportMode::Force);
        assert!(result.is_ok());
        let list = zpool.available_in_dir(PathBuf::from(&vdev_dir)).unwrap();
        assert_eq!(list.len(), 1);

        let result = zpool.import_from_dir(&name, PathBuf::from(vdev_dir));
        assert!(result.is_ok());

        zpool.destroy(&name, DestroyMode::Force).unwrap();

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
        assert_eq!(&result, &topo);
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
        assert_eq!(&result, &topo);
    });
}

#[test]
fn test_all_empty() {
    run_test(|_name| {
        let zpool = ZpoolOpen3::default();

        let result = zpool.all().unwrap();
        assert!(result.is_empty());
    });
}

#[test]
fn test_zpool_with_logger() { let _zpool = ZpoolOpen3::with_logger(get_logger()); }

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

        let result = zpool.take_offline(&name, &vdev_path, OfflineMode::UntilReboot);
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
            .vdev(CreateVdevRequest::Mirror(vec![vdev0_path.clone(), vdev1_path.clone()]))
            .build()
            .unwrap();
        zpool.create(topo).unwrap();
        let result = zpool.take_offline(&name, &vdev0_path, OfflineMode::UntilReboot);
        assert!(result.is_ok());

        let z = zpool.status(&name).unwrap();
        assert_eq!(&Health::Degraded, z.health());

        let result = zpool.bring_online(&name, &vdev0_path, OnlineMode::Simple);
        assert!(result.is_ok());

        let z = zpool.status(&name).unwrap();
        assert_eq!(&Health::Online, z.health());
    });
}

#[test]
fn test_zpool_take_device_from_mirror_offline_expand() {
    run_test(|name| {
        let zpool = ZpoolOpen3::default();
        let vdev0_path = setup_vdev("/vdevs/vdev3", &Bytes::MegaBytes(64 + 10));
        let vdev1_path = setup_vdev("/vdevs/vdev4", &Bytes::MegaBytes(64 + 10));
        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::Mirror(vec![vdev0_path.clone(), vdev1_path.clone()]))
            .build()
            .unwrap();
        zpool.create(topo).unwrap();
        let result = zpool.take_offline(&name, &vdev0_path, OfflineMode::UntilReboot);
        assert!(result.is_ok());

        let z = zpool.status(&name).unwrap();
        assert_eq!(&Health::Degraded, z.health());

        let result = zpool.bring_online(&name, &vdev0_path, OnlineMode::Expand);
        assert!(result.is_ok());

        let z = zpool.status(&name).unwrap();
        assert_eq!(&Health::Online, z.health());
    });
}

#[test]
fn test_zpool_attach_then_detach_single() {
    run_test(|name| {
        let zpool = ZpoolOpen3::default();
        let vdev0_path = setup_vdev("/vdevs/vdev1", &Bytes::MegaBytes(64 + 10));
        let vdev1_path = setup_vdev("/vdevs/vdev2", &Bytes::MegaBytes(64 + 10));
        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::SingleDisk(vdev0_path.clone()))
            .build()
            .unwrap();
        zpool.create(topo.clone()).unwrap();

        zpool.attach(&name, &vdev0_path, &vdev1_path).unwrap();

        let z = zpool.status(&name).unwrap();
        let topo_actual = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::Mirror(vec![vdev0_path.clone(), vdev1_path.clone()]))
            .build()
            .unwrap();
        assert_eq!(&z, &topo_actual);

        zpool.detach(&name, &vdev1_path).unwrap();
        let z = zpool.status(&name).unwrap();
        assert_eq!(&z, &topo);

        let err = zpool.detach(&name, &vdev0_path).unwrap_err();
        assert_eq!(ZpoolErrorKind::OnlyDevice, err.kind());
    });
}

#[test]
fn test_zpool_add_naked() {
    run_test(|name| {
        let zpool = ZpoolOpen3::default();
        let vdev0_path = setup_vdev("/vdevs/vdev1", &Bytes::MegaBytes(64 + 10));
        let vdev1_path = setup_vdev("/vdevs/vdev2", &Bytes::MegaBytes(64 + 10));
        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::SingleDisk(vdev0_path.clone()))
            .build()
            .unwrap();
        zpool.create(topo.clone()).unwrap();

        let new_vdev = CreateVdevRequest::SingleDisk(vdev1_path.clone());

        let result = zpool.add_vdev(&name, new_vdev, CreateMode::Gentle);

        let topo_expected = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::SingleDisk(vdev0_path.clone()))
            .vdev(CreateVdevRequest::SingleDisk(vdev1_path.clone()))
            .build()
            .unwrap();

        assert!(result.is_ok());

        let z = zpool.status(&name).unwrap();

        assert_eq!(topo_expected, z);
    });
}
#[test]
fn test_zpool_add_naked_force() {
    run_test(|name| {
        let zpool = ZpoolOpen3::default();
        let vdev0_path = setup_vdev("/vdevs/vdev1", &Bytes::MegaBytes(64 + 10));
        let vdev1_path = setup_vdev("/vdevs/vdev2", &Bytes::MegaBytes(64 + 10));
        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::SingleDisk(vdev0_path.clone()))
            .build()
            .unwrap();
        zpool.create(topo.clone()).unwrap();

        let new_vdev = CreateVdevRequest::SingleDisk(vdev1_path.clone());

        let result = zpool.add_vdev(&name, new_vdev, CreateMode::Force);

        let topo_expected = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::SingleDisk(vdev0_path.clone()))
            .vdev(CreateVdevRequest::SingleDisk(vdev1_path.clone()))
            .build()
            .unwrap();

        assert!(result.is_ok());

        let z = zpool.status(&name).unwrap();

        assert_eq!(topo_expected, z);
    });
}
#[test]
fn test_zpool_add_mirror() {
    run_test(|name| {
        let zpool = ZpoolOpen3::default();
        let vdev0_path = setup_vdev("/vdevs/vdev1", &Bytes::MegaBytes(64 + 10));
        let vdev1_path = setup_vdev("/vdevs/vdev2", &Bytes::MegaBytes(64 + 10));
        let vdev2_path = setup_vdev("/vdevs/vdev3", &Bytes::MegaBytes(64 + 10));
        let vdev3_path = setup_vdev("/vdevs/vdev4", &Bytes::MegaBytes(64 + 10));
        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::Mirror(vec![vdev0_path.clone(), vdev1_path.clone()]))
            .build()
            .unwrap();
        zpool.create(topo.clone()).unwrap();

        let new_vdev = CreateVdevRequest::Mirror(vec![vdev2_path.clone(), vdev3_path.clone()]);

        let result = zpool.add_vdev(&name, new_vdev, CreateMode::default());

        let topo_expected = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::Mirror(vec![vdev0_path.clone(), vdev1_path.clone()]))
            .vdev(CreateVdevRequest::Mirror(vec![vdev2_path.clone(), vdev3_path.clone()]))
            .build()
            .unwrap();

        assert!(result.is_ok());

        let z = zpool.status(&name).unwrap();

        assert_eq!(topo_expected, z);
    });
}

// Somehow this is okay on ZOL, but not okay on FreeBSD.
#[test]
#[cfg(target_os = "freebsd")]
fn test_zpool_add_mirror_to_raidz() {
    run_test(|name| {
        let zpool = ZpoolOpen3::default();
        let vdev0_path = setup_vdev("/vdevs/vdev1", &Bytes::MegaBytes(64 + 10));
        let vdev1_path = setup_vdev("/vdevs/vdev2", &Bytes::MegaBytes(64 + 10));
        let vdev2_path = setup_vdev("/vdevs/vdev3", &Bytes::MegaBytes(64 + 10));
        let vdev3_path = setup_vdev("/vdevs/vdev4", &Bytes::MegaBytes(64 + 10));
        let vdev4_path = setup_vdev("/vdevs/vdev5", &Bytes::MegaBytes(64 + 10));
        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::RaidZ(vec![
                vdev0_path.clone(),
                vdev1_path.clone(),
                vdev2_path.clone(),
            ]))
            .build()
            .unwrap();
        zpool.create(topo.clone()).unwrap();

        let new_vdev = CreateVdevRequest::Mirror(vec![vdev3_path.clone(), vdev4_path.clone()]);

        let result = zpool.add_vdev(&name, new_vdev, CreateMode::default());

        assert!(result.is_err());

        if let Err(r) = result {
            assert_eq!(ZpoolErrorKind::MismatchedReplicationLevel, r.kind());
        }
    });
}

#[test]
fn test_zpool_remove_zil() {
    run_test(|name| {
        let zpool = ZpoolOpen3::default();
        let vdev0_path = setup_vdev("/vdevs/vdev1", &Bytes::MegaBytes(64 + 10));
        let vdev1_path = setup_vdev("/vdevs/vdev2", &Bytes::MegaBytes(64 + 10));
        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::SingleDisk(vdev0_path.clone()))
            .zil(CreateVdevRequest::SingleDisk(vdev1_path.clone()))
            .build()
            .unwrap();
        zpool.create(topo).unwrap();

        let result = zpool.remove(&name, &vdev1_path);
        assert!(result.is_ok());

        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::SingleDisk(vdev0_path.clone()))
            .build()
            .unwrap();

        let result = zpool.status(&name).unwrap();

        assert_eq!(topo, result);
    });
}

#[test]
fn test_zpool_add_cache() {
    run_test(|name| {
        let zpool = ZpoolOpen3::default();
        let vdev1_path = setup_vdev("/vdevs/vdev1", &Bytes::MegaBytes(64 + 10));
        let vdev2_path = get_virtual_device();
        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::SingleDisk(vdev1_path.clone()))
            .build()
            .unwrap();
        zpool.create(topo.clone()).unwrap();

        let result = zpool.add_cache(&name, &vdev2_path, CreateMode::Gentle);

        let topo_expected = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::SingleDisk(vdev1_path.clone()))
            .cache(vdev2_path.clone())
            .build()
            .unwrap();

        assert!(result.is_ok());

        let z = zpool.status(&name).unwrap();
        assert_eq!(topo_expected, z);
    });
}

#[test]
fn test_create_with_spare() {
    run_test(|name| {
        let zpool = ZpoolOpen3::default();
        let vdev0_path = setup_vdev("/vdevs/vdev0", &Bytes::MegaBytes(64 + 10));
        let vdev1_path = setup_vdev("/vdevs/vdev1", &Bytes::MegaBytes(64 + 10));

        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::SingleDisk(vdev0_path.clone()))
            .spare(vdev1_path.clone())
            .build()
            .unwrap();
        dbg!(&topo);
        zpool.create(topo.clone()).unwrap();
        let z = zpool.status(&name).unwrap();
        dbg!(&z);
        assert_eq!(topo, z);
    });
}

#[test]
fn test_zpool_add_spare() {
    run_test(|name| {
        let zpool = ZpoolOpen3::default();
        let vdev1_path = setup_vdev("/vdevs/vdev1", &Bytes::MegaBytes(64 + 10));
        let vdev2_path = get_virtual_device();
        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::SingleDisk(vdev1_path.clone()))
            .build()
            .unwrap();
        zpool.create(topo.clone()).unwrap();

        let result = zpool.add_spare(&name, &vdev2_path, CreateMode::Gentle);

        let topo_expected = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::SingleDisk(vdev1_path.clone()))
            .spare(vdev2_path.clone())
            .build()
            .unwrap();

        assert!(result.is_ok());

        let z = zpool.status(&name).unwrap();
        assert_eq!(topo_expected, z);
    });
}

#[test]
fn test_zpool_replace_disk() {
    use std::{thread, time};

    run_test(|name| {
        let zpool = ZpoolOpen3::default();
        let vdev0_path = setup_vdev("/vdevs/vdev0", &Bytes::MegaBytes(64 + 10));
        let vdev1_path = setup_vdev("/vdevs/vdev1", &Bytes::MegaBytes(64 + 10));
        let vdev2_path = setup_vdev("/vdevs/vdev2", &Bytes::MegaBytes(64 + 10));
        let topo = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::Mirror(vec![vdev0_path.clone(), vdev1_path.clone()]))
            .build()
            .unwrap();
        zpool.create(topo.clone()).unwrap();

        let result = zpool.replace_disk(&name, &vdev0_path, &vdev2_path);
        assert!(result.is_ok());

        let topo_expected = CreateZpoolRequestBuilder::default()
            .name(name.clone())
            .create_mode(CreateMode::Force)
            .vdev(CreateVdevRequest::Mirror(vec![vdev2_path.clone(), vdev1_path.clone()]))
            .build()
            .unwrap();

        // otherwise test _might_ fail.
        let wait_time = time::Duration::from_secs(13);
        thread::sleep(wait_time);

        let z = zpool.status(&name).unwrap();
        assert_eq!(topo_expected, z);
    });
}
