use std::{os::unix::io::AsRawFd, path::PathBuf};

use bitflags::bitflags;

pub mod description;
pub use description::DatasetKind;

pub mod delegating;
pub use delegating::DelegatingZfsEngine;
pub mod open3;
pub use open3::ZfsOpen3;

pub mod lzc;
use crate::zfs::properties::{AclInheritMode, AclMode};
pub use lzc::ZfsLzc;
use std::collections::HashMap;

pub mod properties;
pub use properties::{CacheMode, CanMount, Checksum, Compression, Copies, FilesystemProperties,
                     Properties, SnapDir, VolumeProperties};

mod pathext;
pub use pathext::PathExt;

pub static DATASET_NAME_MAX_LENGTH: usize = 255;

mod errors;

pub use errors::{Error, ErrorKind, Result, ValidationError, ValidationResult};

/// Whether to mark busy snapshots for deferred destruction rather than immediately failing if can't
/// be destroyed right now.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DestroyTiming {
    /// If a snapshot has user holds or clones, destroy operation will fail and none of the
    /// snapshots will be destroyed.
    RightNow,
    /// If a snapshot has user holds or clones, it will be marked for deferred destruction, and
    /// will be destroyed when the last hold or clone is removed/destroyed.
    Defer,
}

impl DestroyTiming {
    pub fn as_c_uint(&self) -> std::os::raw::c_uint {
        match self {
            DestroyTiming::Defer => 1,
            DestroyTiming::RightNow => 0,
        }
    }
}

pub struct BookmarkRequest {
    pub snapshot: PathBuf,
    pub bookmark: PathBuf,
}

impl BookmarkRequest {
    pub fn new(snapshot: PathBuf, bookmark: PathBuf) -> Self {
        BookmarkRequest { snapshot, bookmark }
    }
}

bitflags! {
    #[derive(Default)]
    pub struct SendFlags: u32 {
        const LZC_SEND_FLAG_EMBED_DATA = 1 << 0;
        const LZC_SEND_FLAG_LARGE_BLOCK = 1 << 1;
        const LZC_SEND_FLAG_COMPRESS = 1 << 2;
        const LZC_SEND_FLAG_RAW = 1 << 3;
        const LZC_SEND_FLAG_SAVED = 1 << 4;
    }
}
pub trait ZfsEngine {
    /// Check if a dataset (a filesystem, or a volume, or a snapshot with the given name exists.
    ///
    /// NOTE: Can't be used to check for existence of bookmarks.
    ///  * `name` - The dataset name to check.
    #[cfg_attr(tarpaulin, skip)]
    fn exists<N: Into<PathBuf>>(&self, _name: N) -> Result<bool> { Err(Error::Unimplemented) }

    /// Create a new dataset.
    #[cfg_attr(tarpaulin, skip)]
    fn create(&self, _request: CreateDatasetRequest) -> Result<()> { Err(Error::Unimplemented) }

    /// Create snapshots as one atomic operation.
    #[cfg_attr(tarpaulin, skip)]
    fn snapshot(
        &self,
        _snapshots: &[PathBuf],
        _user_properties: Option<HashMap<String, String>>,
    ) -> Result<()> {
        Err(Error::Unimplemented)
    }

    /// Create bookmarks as one atomic operation.
    #[cfg_attr(tarpaulin, skip)]
    fn bookmark(&self, _snapshots: &[BookmarkRequest]) -> Result<()> { Err(Error::Unimplemented) }

    /// Deletes the dataset
    /// Deletes the dataset
    #[cfg_attr(tarpaulin, skip)]
    fn destroy<N: Into<PathBuf>>(&self, _name: N) -> Result<()> { Err(Error::Unimplemented) }

    /// Delete snapshots as one atomic operation
    #[cfg_attr(tarpaulin, skip)]
    fn destroy_snapshots(&self, _snapshots: &[PathBuf], _timing: DestroyTiming) -> Result<()> {
        Err(Error::Unimplemented)
    }

    /// Delete bookmarks as one atomic operation
    #[cfg_attr(tarpaulin, skip)]
    fn destroy_bookmarks(&self, _bookmarks: &[PathBuf]) -> Result<()> { Err(Error::Unimplemented) }

    #[cfg_attr(tarpaulin, skip)]
    fn list<N: Into<PathBuf>>(&self, _pool: N) -> Result<Vec<(DatasetKind, PathBuf)>> {
        Err(Error::Unimplemented)
    }
    #[cfg_attr(tarpaulin, skip)]
    fn list_filesystems<N: Into<PathBuf>>(&self, _pool: N) -> Result<Vec<PathBuf>> {
        Err(Error::Unimplemented)
    }
    #[cfg_attr(tarpaulin, skip)]
    fn list_snapshots<N: Into<PathBuf>>(&self, _pool: N) -> Result<Vec<PathBuf>> {
        Err(Error::Unimplemented)
    }
    #[cfg_attr(tarpaulin, skip)]
    fn list_bookmarks<N: Into<PathBuf>>(&self, _pool: N) -> Result<Vec<PathBuf>> {
        Err(Error::Unimplemented)
    }
    #[cfg_attr(tarpaulin, skip)]
    fn list_volumes<N: Into<PathBuf>>(&self, _pool: N) -> Result<Vec<PathBuf>> {
        Err(Error::Unimplemented)
    }
    /// Read all properties of filesystem/volume/snapshot/bookmark.
    #[cfg_attr(tarpaulin, skip)]
    fn read_properties<N: Into<PathBuf>>(&self, _path: N) -> Result<Properties> {
        Err(Error::Unimplemented)
    }

    /// Send a full snapshot to a specified file descriptor.
    #[cfg_attr(tarpaulin, skip)]
    fn send_full<N: Into<PathBuf>, FD: AsRawFd>(
        &self,
        _path: N,
        _fd: FD,
        _flags: SendFlags,
    ) -> Result<()> {
        Err(Error::Unimplemented)
    }

    /// Send an incremental snapshot to a specified file descriptor.
    #[cfg_attr(tarpaulin, skip)]
    fn send_incremental<N: Into<PathBuf>, F: Into<PathBuf>, FD: AsRawFd>(
        &self,
        _path: N,
        _from: F,
        _fd: FD,
        _flags: SendFlags,
    ) -> Result<()> {
        Err(Error::Unimplemented)
    }

    /// Run a channel program
    #[cfg_attr(tarpaulin, skip)]
    fn run_channel_program<N: Into<PathBuf>>(
        &self,
        _pool: N,
        _program: &str,
        _instr_limit: u64,
        _mem_limit: u64,
        _sync: bool,
        _args: libnv::nvpair::NvList,
    ) -> Result<libnv::nvpair::NvList> {
        Err(Error::Unimplemented)
    }
}

#[derive(Default, Builder, Debug, Clone, Getters)]
#[builder(setter(into))]
#[get = "pub"]
/// Consumer friendly builder for NvPair. Use this to create your datasets. Some properties only
/// work on filesystems, some only on volumes.
pub struct CreateDatasetRequest {
    /// Name of the dataset. First crumb of path is name of zpool.
    name:            PathBuf,
    /// Filesystem or Volume.
    kind:            DatasetKind,
    /// Optional user defined properties. User property names must conform to the following
    /// characteristics:
    ///
    ///  - Contain a colon (':') character to distinguish them from native properties.
    ///  - Contain lowercase letters, numbers, and the following punctuation characters: ':',
    ///    '+','.', '_'.
    ///  - Maximum user property name is 256 characters.
    #[builder(default)]
    user_properties: Option<HashMap<String, String>>,

    //
    // the rest is zfs native properties
    /// Controls how ACL entries inherited when files and directories created.
    #[builder(default)]
    acl_inherit:       Option<AclInheritMode>,
    /// Controls how an ACL entry modified during a `chmod` operation.
    #[builder(default)]
    acl_mode:          Option<AclMode>,
    /// Controls whether the access time for files updated when they are read.
    #[builder(default)]
    atime:             Option<bool>,
    /// Controls whether a file system can be mounted.
    #[builder(default)]
    can_mount:         CanMount,
    /// Controls the checksum used to verify data integrity.
    #[builder(default)]
    checksum:          Option<Checksum>,
    /// Enables or disables compression for a dataset.
    #[builder(default)]
    compression:       Option<Compression>,
    /// Sets the number of copies of user data per file system. Available values are 1, 2, or 3.
    /// These copies are in addition to any pool-level redundancy. Disk space used by multiple
    /// copies of user data charged to the corresponding file and dataset, and counts against
    /// quotas and reservations. In addition, the used property updated when multiple copies
    /// enabled. Consider setting this property when the file system created because changing this
    /// property on an existing file system only affects newly written data.
    #[builder(default)]
    copies:            Option<Copies>,
    /// Controls whether device files in a file system can be opened.
    #[builder(default)]
    devices:           Option<bool>,
    /// Controls whether programs in a file system allowed to be executed. Also, when set to
    /// `false`, `mmap(2)` calls with `PROT_EXEC` disallowed.
    #[builder(default)]
    exec:              Option<bool>,
    /// Controls the mount point used for this file system.
    #[builder(default)]
    mount_point:       Option<PathBuf>,
    /// Controls what is cached in the primary cache (ARC).
    #[builder(default)]
    primary_cache:     Option<CacheMode>,
    /// Limits the amount of disk space a dataset and its descendants can consume.
    #[builder(default)]
    quota:             Option<u64>,
    /// Controls whether a dataset can be modified.
    #[builder(default)]
    readonly:          Option<bool>,
    /// Specifies a suggested block size for files in a file system in bytes. The size specified
    /// must be a power of two greater than or equal to 512 and less than or equal to 128 KiB.
    /// If the large_blocks feature is enabled on the pool, the size may be up to 1 MiB.
    #[builder(default)]
    record_size:       Option<u64>,
    /// Sets the amount of disk space a dataset can consume. This property enforces a hard limit on
    /// the amount of space used. This hard limit does not include disk space used by descendents,
    /// such as snapshots and clones.
    #[builder(default)]
    ref_quota:         Option<u64>,
    /// Sets the minimum amount of disk space is guaranteed to a dataset, not including
    /// descendants, such as snapshots and clones.
    #[builder(default)]
    ref_reservation:   Option<u64>,
    /// Sets the minimum amount of disk space guaranteed to a dataset and its descendants.
    #[builder(default)]
    reservation:       Option<u64>,
    /// Controls what is cached in the secondary cache (L2ARC).
    #[builder(default)]
    secondary_cache:   Option<CacheMode>,
    /// Controls whether the `setuid` bit is honored in a file system.
    #[builder(default)]
    setuid:            Option<bool>,
    /// Controls whether the .zfs directory is hidden or visible in the root of the file system
    #[builder(default)]
    snap_dir:          Option<SnapDir>,
    /// For volumes, specifies the logical size of the volume.
    #[builder(default)]
    volume_size:       Option<u64>,
    /// For volumes, specifies the block size of the volume in bytes. The block size cannot be
    /// changed after the volume has been written, so set the block size at volume creation time.
    /// The default block size for volumes is 8 KB. Any power of 2 from 512 bytes to 128 KB is
    /// valid.
    #[builder(default)]
    volume_block_size: Option<u64>,
    /// Indicates whether extended attributes are enabled or disabled.
    #[builder(default)]
    xattr:             Option<bool>,
}

impl CreateDatasetRequest {
    pub fn builder() -> CreateDatasetRequestBuilder { CreateDatasetRequestBuilder::default() }

    pub fn validate(&self) -> Result<()> {
        let mut errors = Vec::new();

        if let Err(e) = validators::validate_name(self.name()) {
            errors.push(e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.into())
        }
    }
}

pub(crate) mod validators {
    use crate::zfs::{errors::ValidationResult, ValidationError, DATASET_NAME_MAX_LENGTH};
    use std::path::Path;

    pub fn validate_name<P: AsRef<Path>>(dataset: P) -> ValidationResult {
        _validate_name(dataset.as_ref())
    }

    pub fn _validate_name(dataset: &Path) -> ValidationResult {
        let name = dataset.to_string_lossy();
        if name.ends_with('/') {
            return Err(ValidationError::MissingName(dataset.to_owned()));
        }
        if dataset.has_root() {
            return Err(ValidationError::MissingPool(dataset.to_owned()));
        }
        dataset
            .file_name()
            .ok_or_else(|| ValidationError::MissingName(dataset.to_owned()))
            .and_then(|name| {
                if name.len() > DATASET_NAME_MAX_LENGTH {
                    return Err(ValidationError::NameTooLong(dataset.to_owned()));
                }
                Ok(())
            })
    }
}

#[cfg(test)]
mod test {
    use super::{CreateDatasetRequest, DatasetKind, Error, ErrorKind, ValidationError};
    use std::path::PathBuf;

    #[test]
    fn test_error_ds_not_found() {
        let stderr = b"cannot open 's/asd/asd': dataset does not exist";

        let err = Error::from_stderr(stderr);
        assert_eq!(Error::DatasetNotFound(PathBuf::from("s/asd/asd")), err);
        assert_eq!(ErrorKind::DatasetNotFound, err.kind());
    }

    #[test]
    fn test_error_rubbish() {
        let stderr = b"there is no way there is an error like this";
        let stderr_string = String::from_utf8_lossy(stderr).to_string();

        let err = Error::from_stderr(stderr);
        assert_eq!(Error::UnknownSoFar(stderr_string), err);
        assert_eq!(ErrorKind::Unknown, err.kind());
    }

    #[test]
    fn test_name_validator() {
        let path = PathBuf::from("z/asd/");
        let request = CreateDatasetRequest::builder()
            .name(path.clone())
            .kind(DatasetKind::Filesystem)
            .build()
            .unwrap();

        let result = request.validate().unwrap_err();
        let expected = Error::from(vec![ValidationError::MissingName(path.clone())]);
        assert_eq!(expected, result);

        let path = PathBuf::from("z/asd/jnmgyfklueiodyfryvopvyfidvdgxqxsesjmqeoevdgmzsqmesuqzqoxhjfltmsvltdyiilgkvklinlfhaanfqisdazjpfmwttnuosdfijickudhwegburxsoesvunamysaigtagymxcyfeyqiqphtalmbkskrjdndbbcjqiiwucsxzezqmvpzmkylrojumtvatfvrpfkxubfujyioyylmffvrvtfetnzghkwaqzxkqmialkaaekotuhgiivwvbsoqqa");
        let request = CreateDatasetRequest::builder()
            .name(path.clone())
            .kind(DatasetKind::Filesystem)
            .build()
            .unwrap();

        let result = request.validate().unwrap_err();
        let expected = Error::from(vec![ValidationError::NameTooLong(path.clone())]);
        assert_eq!(expected, result);
    }
}
