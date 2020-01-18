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
               zfs::{Copies, CreateDatasetRequest, DatasetKind, Error,
                     ZfsEngine, ZfsLzc, SnapDir, Properties},
               zpool::{CreateVdevRequest, CreateZpoolRequest, ZpoolEngine, ZpoolOpen3}};

use libzetta::{zfs::DelegatingZfsEngine, zpool::CreateMode};
use libzetta::zfs::DestroyTiming;

static ONE_MB_IN_BYTES: u64 = 1024 * 1024;

static ZPOOL_NAME_PREFIX: &'static str = "tests-zfs-";
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

    let zfs = ZfsLzc::new(None).expect("Failed to initialize ZfsLzc");

    let result = zfs.exists(fake_dataset).unwrap();

    assert!(!result);
}

#[test]
fn create_dumb() {
    let zpool = SHARED_ZPOOL.clone();
    let dataset_path = PathBuf::from(format!("{}/{}", zpool, get_dataset_name()));

    let zfs = ZfsLzc::new(None).expect("Failed to initialize ZfsLzc");

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

    let zfs = ZfsLzc::new(None).expect("Failed to initialize ZfsLzc");

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
        .name(dataset_path.clone())
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

    let zfs = DelegatingZfsEngine::new(None).expect("Failed to initialize ZfsLzc");
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
    let zfs = DelegatingZfsEngine::new(None).expect("Failed to initialize ZfsLzc");
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
    let datasets = zfs.list(root.clone()).unwrap();
    assert_eq!(5, datasets.len());
    assert_eq!(expected, datasets);
}

#[test]
fn easy_snapshot() {
    let zpool = SHARED_ZPOOL.clone();
    let zfs = DelegatingZfsEngine::new(None).expect("Failed to initialize ZfsLzc");
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

    let snapshots =
        zfs.list_snapshots(PathBuf::from(root.clone())).expect("failed to list snapshots");
    assert_eq!(expected_snapshots, snapshots);

    assert_eq!(Ok(true), zfs.exists(expected_snapshots[0].clone()));

    zfs.destroy_snapshots(&expected_snapshots, DestroyTiming::RightNow).unwrap();
    assert_eq!(Ok(false), zfs.exists(expected_snapshots[0].clone()));

}


#[test]
fn read_properties() {
    let zpool = SHARED_ZPOOL.clone();
    let zfs = DelegatingZfsEngine::new(None).expect("Failed to initialize ZfsLzc");
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
    let test = zfs.read_properties(&root).unwrap();
    if let Properties::Filesystem(properties) = zfs.read_properties(&root).unwrap() {
        assert_eq!(&SnapDir::Visible, properties.snap_dir());
        assert_eq!(&Copies::Two, properties.copies());
    } else {
        panic!("Read not fs properties");
    }

}