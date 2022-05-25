#![allow(clippy::mutex_atomic)]
#[macro_use] extern crate lazy_static;

use std::{fs::{self, DirBuilder},
          panic,
          path::{Path, PathBuf},
          sync::Mutex};

use cavity::{fill, Bytes, WriteMode};
use rand::Rng;

use libzetta::{slog::*,
               zfs::{BookmarkRequest, Copies, CreateDatasetRequest, DatasetKind, Error,
                     Properties, SendFlags, SnapDir, ZfsEngine, ZfsLzc},
               zpool::{CreateVdevRequest, CreateZpoolRequest, ZpoolEngine, ZpoolOpen3}};

use libzetta::{zfs::{properties::VolumeMode, DelegatingZfsEngine, DestroyTiming},
               zpool::CreateMode};

static ONE_MB_IN_BYTES: u64 = 1024 * 1024;

static ZPOOL_NAME_PREFIX: &str = "tests-zfs-";
lazy_static! {
    static ref INITIALIZED: Mutex<bool> = Mutex::new(false);
    static ref SHARED_ZPOOL: String = {
        let name = get_zpool_name();
        setup_zpool(&name);
        name
    };
}
fn get_zpool_name() -> String {
    let mut rng = rand::thread_rng();
    let suffix = rng.gen::<u64>();
    let name = format!("{}-{}", ZPOOL_NAME_PREFIX, suffix);
    name
}
fn get_dataset_name() -> String {
    let mut rng = rand::thread_rng();
    let name = rng.gen::<u64>();
    let name = format!("{}", name);
    name
}

fn setup_zpool(name: &str) {
    let data = INITIALIZED.lock().unwrap();

    if !*data {
        // Create vdevs if they're missing
        let vdev_dir = Path::new("/vdevs/zfs");
        setup_vdev(vdev_dir.join("vdev0"), &Bytes::MegaBytes(64 + 10));
        let zpool = ZpoolOpen3::default();
        let topo = CreateZpoolRequest::builder()
            .name(name)
            .vdev(CreateVdevRequest::SingleDisk("/vdevs/zfs/vdev0".into()))
            .create_mode(CreateMode::Force)
            .build()
            .unwrap();
        zpool.create(topo).unwrap();
    }
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
// Only used for debugging
#[allow(dead_code)]
fn get_logger() -> Option<Logger> {
    let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());
    Some(Logger::root(slog_term::FullFormat::new(plain).use_original_order().build().fuse(), o!()))
}

#[test]
fn exists_on_fake() {
    let zpool = SHARED_ZPOOL.clone();
    let fake_dataset = format!("{}/very/fake/dataset", zpool);

    let zfs = ZfsLzc::new().expect("Failed to initialize ZfsLzc");

    let result = zfs.exists(fake_dataset).unwrap();

    assert!(!result);
}

#[test]
fn create_dumb() {
    let zpool = SHARED_ZPOOL.clone();
    let dataset_path = PathBuf::from(format!("{}/{}", zpool, get_dataset_name()));

    let zfs = ZfsLzc::new().expect("Failed to initialize ZfsLzc");

    let request = CreateDatasetRequest::builder()
        .name(dataset_path.clone())
        .user_properties(std::collections::HashMap::new())
        .kind(DatasetKind::Filesystem)
        .copies(Copies::Three)
        .build()
        .unwrap();

    zfs.create(request).expect("Failed to create dataset");

    let res = zfs.exists(dataset_path.to_str().unwrap()).unwrap();
    assert!(res);
}

#[test]
fn easy_invalid_zfs() {
    let zpool = SHARED_ZPOOL.clone();
    let dataset_path = PathBuf::from(format!("{}/{}", zpool, get_dataset_name()));

    let zfs = ZfsLzc::new().expect("Failed to initialize ZfsLzc");

    let request = CreateDatasetRequest::builder()
        .name(dataset_path.clone())
        .user_properties(std::collections::HashMap::new())
        .kind(DatasetKind::Filesystem)
        .volume_size(2)
        .build()
        .unwrap();

    let res = zfs.create(request).unwrap_err();
    assert_eq!(Error::invalid_input(), res);

    let request = CreateDatasetRequest::builder()
        .name(dataset_path.clone())
        .user_properties(std::collections::HashMap::new())
        .kind(DatasetKind::Filesystem)
        .volume_block_size(2)
        .build()
        .unwrap();

    let res = zfs.create(request).unwrap_err();
    assert_eq!(Error::invalid_input(), res);

    let request = CreateDatasetRequest::builder()
        .name(dataset_path.clone())
        .user_properties(std::collections::HashMap::new())
        .kind(DatasetKind::Filesystem)
        .volume_size(2)
        .volume_block_size(2)
        .build()
        .unwrap();

    let res = zfs.create(request).unwrap_err();
    assert_eq!(Error::invalid_input(), res);

    let request = CreateDatasetRequest::builder()
        .name(dataset_path)
        .user_properties(std::collections::HashMap::new())
        .kind(DatasetKind::Volume)
        .build()
        .unwrap();

    let res = zfs.create(request).unwrap_err();
    assert_eq!(Error::invalid_input(), res);
}

#[test]
fn create_and_destroy() {
    let zpool = SHARED_ZPOOL.clone();
    let dataset_path = PathBuf::from(format!("{}/{}", zpool, get_dataset_name()));

    let zfs = DelegatingZfsEngine::new().expect("Failed to initialize ZfsLzc");
    let request = CreateDatasetRequest::builder()
        .name(dataset_path.clone())
        .user_properties(std::collections::HashMap::new())
        .kind(DatasetKind::Filesystem)
        .build()
        .unwrap();

    zfs.create(request).expect("Failed to create the dataset");

    let res = zfs.exists(dataset_path.to_str().unwrap()).unwrap();
    assert!(res);

    zfs.destroy(dataset_path.clone()).unwrap();
    let res = zfs.exists(dataset_path.to_str().unwrap()).unwrap();
    assert!(!res);
}

#[test]
fn create_and_list() {
    let zpool = SHARED_ZPOOL.clone();
    let zfs = DelegatingZfsEngine::new().expect("Failed to initialize ZfsLzc");
    let root = PathBuf::from(format!("{}/{}", zpool, get_dataset_name()));
    let mut expected_filesystems = vec![root.clone()];
    let mut expected_volumes = Vec::with_capacity(2);
    let request = CreateDatasetRequest::builder()
        .name(root.clone())
        .kind(DatasetKind::Filesystem)
        .build()
        .unwrap();
    zfs.create(request).expect("Failed to create a root dataset");

    for idx in 0..2 {
        let mut path = root.clone();
        path.push(format!("{}", idx));
        expected_filesystems.push(path.clone());
        let request = CreateDatasetRequest::builder()
            .name(path)
            .kind(DatasetKind::Filesystem)
            .build()
            .unwrap();
        zfs.create(request).expect("Failed to create a dataset");
    }
    let datasets = zfs.list_filesystems(root.clone()).unwrap();
    assert_eq!(3, datasets.len());
    assert_eq!(expected_filesystems, datasets);

    for idx in 2..4 {
        let mut path = root.clone();
        path.push(format!("{}", idx));
        expected_volumes.push(path.clone());
        let request = CreateDatasetRequest::builder()
            .name(path)
            .kind(DatasetKind::Volume)
            .volume_size(ONE_MB_IN_BYTES)
            .build()
            .unwrap();
        zfs.create(request).expect("Failed to create a dataset");
    }
    let datasets = zfs.list_volumes(root.clone()).unwrap();
    assert_eq!(2, datasets.len());
    assert_eq!(expected_volumes, datasets);
    let expected: Vec<(DatasetKind, PathBuf)> = expected_filesystems
        .into_iter()
        .map(|e| (DatasetKind::Filesystem, e))
        .chain(expected_volumes.into_iter().map(|e| (DatasetKind::Volume, e)))
        .collect();
    let datasets = zfs.list(root).unwrap();
    assert_eq!(5, datasets.len());
    assert_eq!(expected, datasets);
}

#[test]
fn easy_snapshot_and_bookmark() {
    let zpool = SHARED_ZPOOL.clone();
    let zfs = DelegatingZfsEngine::new().expect("Failed to initialize ZfsLzc");
    let root_name = get_dataset_name();
    let root = PathBuf::from(format!("{}/{}", zpool, &root_name));
    let request = CreateDatasetRequest::builder()
        .name(root.clone())
        .kind(DatasetKind::Filesystem)
        .build()
        .unwrap();
    zfs.create(request).expect("Failed to create a root dataset");
    let expected_snapshots = vec![PathBuf::from(format!("{}/{}@snap-1", zpool, &root_name))];

    zfs.snapshot(&expected_snapshots, None).expect("Failed to create snapshots");

    let snapshots = zfs.list_snapshots(root.clone()).expect("failed to list snapshots");
    assert_eq!(expected_snapshots, snapshots);
    assert_eq!(Ok(true), zfs.exists(expected_snapshots[0].clone()));

    let expected_bookmarks = vec![PathBuf::from(format!("{}/{}#snap-1", zpool, &root_name))];

    let bookmark_requests: Vec<BookmarkRequest> = expected_snapshots
        .iter()
        .zip(expected_bookmarks.iter())
        .map(|(snapshot, bookmark)| BookmarkRequest::new(snapshot.clone(), bookmark.clone()))
        .collect();
    zfs.bookmark(&bookmark_requests).expect("Failed to create bookmarks");

    let bookmarks = zfs.list_bookmarks(root.clone()).expect("failed to list bookmarks");
    assert_eq!(expected_bookmarks, bookmarks);

    zfs.destroy_snapshots(&expected_snapshots, DestroyTiming::RightNow).unwrap();
    assert_eq!(Ok(false), zfs.exists(expected_snapshots[0].clone()));

    zfs.destroy_bookmarks(&expected_bookmarks).unwrap();
    let bookmarks = zfs.list_bookmarks(root).expect("failed to list bookmarks");
    assert!(bookmarks.is_empty())
}

#[test]
fn read_properties_of_filesystem() {
    let zpool = SHARED_ZPOOL.clone();
    let zfs = DelegatingZfsEngine::new().expect("Failed to initialize ZfsLzc");
    let root_name = get_dataset_name();
    let root = PathBuf::from(format!("{}/{}", zpool, &root_name));
    let request = CreateDatasetRequest::builder()
        .name(root.clone())
        .kind(DatasetKind::Filesystem)
        .copies(Copies::Two)
        .snap_dir(SnapDir::Visible)
        .build()
        .unwrap();
    zfs.create(request).expect("Failed to create a root dataset");
    if let Properties::Filesystem(properties) = zfs.read_properties(&root).unwrap() {
        assert_eq!(&SnapDir::Visible, properties.snap_dir());
        assert_eq!(&Copies::Two, properties.copies());
    } else {
        panic!("Read not fs properties");
    }
}

#[test]
#[cfg(target_os = "freebsd")]
fn read_properties_of_snapshot_and_bookmark_blessed_os() {
    let zpool = SHARED_ZPOOL.clone();
    let zfs = DelegatingZfsEngine::new().expect("Failed to initialize ZfsLzc");
    let root_name = get_dataset_name();
    let root = PathBuf::from(format!("{}/{}", zpool, &root_name));
    let request = CreateDatasetRequest::builder()
        .name(root)
        .kind(DatasetKind::Filesystem)
        .copies(Copies::Two)
        .snap_dir(SnapDir::Visible)
        .build()
        .unwrap();
    zfs.create(request).expect("Failed to create a root dataset");

    let snapshot_name = format!("{}/{}@properties", zpool, &root_name);

    zfs.snapshot(&[PathBuf::from(&snapshot_name)], None).expect("Failed to create snapshots");

    if let Properties::Snapshot(properties) = zfs.read_properties(&snapshot_name).unwrap() {
        assert_eq!(&None, properties.clones());
        assert_eq!(&Some(VolumeMode::Default), properties.volume_mode());

        let bookmark_name = format!("{}/{}#properties", zpool, &root_name);
        let bookmark_request =
            BookmarkRequest::new(PathBuf::from(&snapshot_name), PathBuf::from(&bookmark_name));
        zfs.bookmark(&[bookmark_request]).expect("Failed to create snapshots");

        if let Properties::Bookmark(properties_bookmark) =
            zfs.read_properties(&bookmark_name).unwrap()
        {
            assert_eq!(properties.create_txg(), properties_bookmark.create_txg());
            assert_eq!(properties.creation(), properties_bookmark.creation());
        } else {
            panic!("Read wrong properties");
        }
    } else {
        panic!("Read wrong properties");
    }
}
#[test]
fn read_properties_of_snapshot() {
    let zpool = SHARED_ZPOOL.clone();
    let zfs = DelegatingZfsEngine::new().expect("Failed to initialize ZfsLzc");
    let root_name = get_dataset_name();
    let root = PathBuf::from(format!("{}/{}", zpool, &root_name));
    let request = CreateDatasetRequest::builder()
        .name(root)
        .kind(DatasetKind::Filesystem)
        .copies(Copies::Two)
        .snap_dir(SnapDir::Visible)
        .build()
        .unwrap();
    zfs.create(request).expect("Failed to create a root dataset");

    let snapshot_name = format!("{}/{}@properties", zpool, &root_name);

    zfs.snapshot(&[PathBuf::from(&snapshot_name)], None).expect("Failed to create snapshots");

    if let Properties::Snapshot(properties) = zfs.read_properties(&snapshot_name).unwrap() {
        assert_eq!(&None, properties.clones());

        let bookmark_name = format!("{}/{}#properties", zpool, &root_name);
        let bookmark_request =
            BookmarkRequest::new(PathBuf::from(&snapshot_name), PathBuf::from(&bookmark_name));
        zfs.bookmark(&[bookmark_request]).expect("Failed to create snapshots");

        if let Properties::Bookmark(properties_bookmark) =
            zfs.read_properties(&bookmark_name).unwrap()
        {
            assert_eq!(properties.create_txg(), properties_bookmark.create_txg());
            assert_eq!(properties.creation(), properties_bookmark.creation());
        } else {
            panic!("Read wrong properties");
        }
    } else {
        panic!("Read wrong properties");
    }
}
#[test]
fn read_properties_of_volume() {
    let zpool = SHARED_ZPOOL.clone();
    let zfs = DelegatingZfsEngine::new().expect("Failed to initialize ZfsLzc");
    let root_name = get_dataset_name();
    let root = PathBuf::from(format!("{}/{}", zpool, &root_name));
    let request = CreateDatasetRequest::builder()
        .name(root.clone())
        .kind(DatasetKind::Volume)
        .volume_size(ONE_MB_IN_BYTES)
        .build()
        .unwrap();
    zfs.create(request).expect("Failed to create a root dataset");

    if let Properties::Volume(properties) = zfs.read_properties(&root).unwrap() {
        assert_eq!(&root, properties.name());
    } else {
        panic!("Read not fs properties");
    }
}
#[test]
fn send_snapshot() {
    let zpool = SHARED_ZPOOL.clone();
    let zfs = DelegatingZfsEngine::new().expect("Failed to initialize ZfsLzc");
    let root_name = get_dataset_name();
    let root = PathBuf::from(format!("{}/{}", zpool, &root_name));
    let request = CreateDatasetRequest::builder()
        .name(root)
        .kind(DatasetKind::Volume)
        .volume_size(ONE_MB_IN_BYTES)
        .build()
        .unwrap();
    zfs.create(request).expect("Failed to create a root dataset");

    let snapshot_name = format!("{}/{}@tosend", zpool, &root_name);
    let snapshot = PathBuf::from(&snapshot_name);

    zfs.snapshot(&[PathBuf::from(&snapshot_name)], None).expect("Failed to create snapshots");

    let tmpfile = tempfile::tempfile().unwrap();

    zfs.send_full(snapshot, tmpfile, SendFlags::empty()).unwrap();
}
#[test]
fn send_snapshot_incremental() {
    let zpool = SHARED_ZPOOL.clone();
    let zfs = DelegatingZfsEngine::new().expect("Failed to initialize ZfsLzc");
    let root_name = get_dataset_name();
    let root = PathBuf::from(format!("{}/{}", zpool, &root_name));
    let request = CreateDatasetRequest::builder()
        .name(root)
        .kind(DatasetKind::Volume)
        .volume_size(ONE_MB_IN_BYTES)
        .build()
        .unwrap();
    zfs.create(request).expect("Failed to create a root dataset");

    let src_snapshot_name = format!("{}/{}@first", zpool, &root_name);
    let src_snapshot = PathBuf::from(&src_snapshot_name);
    zfs.snapshot(&[PathBuf::from(&src_snapshot_name)], None).expect("Failed to create snapshots");

    let snapshot_name = format!("{}/{}@tosend", zpool, &root_name);
    let snapshot = PathBuf::from(&snapshot_name);
    zfs.snapshot(&[PathBuf::from(&snapshot_name)], None).expect("Failed to create snapshots");


    let tmpfile = tempfile::tempfile().unwrap();

    zfs.send_incremental(snapshot, src_snapshot, tmpfile, SendFlags::empty()).unwrap();
}
