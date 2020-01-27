use crate::zfs::{Checksum, Compression, Copies, CreateDatasetRequest, DatasetKind, DestroyTiming,
                 Error, Result, SnapDir, ValidationError, ZfsEngine};
use cstr_argument::CStrArgument;
use libnv::nvpair::NvList;
use slog::{Drain, Logger};
use slog_stdlog::StdLog;

use crate::zfs::{errors::Error::ValidationErrors,
                 properties::{AclInheritMode, AclMode, ZfsProp},
                 PathExt};
use std::{collections::HashMap, ffi::CString, path::PathBuf, ptr::null_mut};
use zfs_core_sys as sys;

fn setup_logger<L: Into<Logger>>(logger: L) -> Logger {
    logger
        .into()
        .new(o!("zetta_module" => "zfs", "zfs_impl" => "lzc", "zetta_version" => crate::VERSION))
}

#[derive(Debug)]
pub struct ZfsLzc {
    logger: Logger,
}

impl ZfsLzc {
    /// Initialize libzfs_core backed ZfsEngine.
    /// If root logger is None, then StdLog drain used.
    pub fn new(root_logger: Option<Logger>) -> Result<Self> {
        let errno = unsafe { sys::libzfs_core_init() };

        if errno != 0 {
            let io_error = std::io::Error::from_raw_os_error(errno);
            return Err(Error::LZCInitializationFailed(io_error));
        }
        let logger = {
            if let Some(slog) = root_logger {
                setup_logger(slog)
            } else {
                let slog = Logger::root(StdLog.fuse(), o!());
                setup_logger(slog)
            }
        };
        Ok(ZfsLzc { logger })
    }

    pub fn logger(&self) -> &Logger { &self.logger }
}

impl ZfsEngine for ZfsLzc {
    fn exists<N: Into<PathBuf>>(&self, name: N) -> Result<bool> {
        let path = name.into();
        let n = path.to_str().expect("Invalid Path").into_cstr();
        let ret = unsafe { sys::lzc_exists(n.as_ref().as_ptr()) };

        if ret == 1 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn create(&self, request: CreateDatasetRequest) -> Result<()> {
        request.validate()?;

        //let mut props = nvpair::NvList::new()?;
        let mut props = NvList::default();
        let name_c_string =
            CString::new(request.name().to_str().expect("Non UTF-8 name")).expect("NULL in name");
        // LZC wants _everything_ as u64 even booleans.
        props.insert_u64(AclInheritMode::nv_key(), request.acl_inherit.as_nv_value())?;
        if let Some(acl_mode) = request.acl_mode {
            props.insert_u64(AclMode::nv_key(), acl_mode.as_nv_value())?;
        }
        props.insert_u64("atime", bool_to_u64(request.atime))?;
        props.insert_u64(Checksum::nv_key(), request.checksum.as_nv_value())?;
        props.insert_u64(Compression::nv_key(), request.compression.as_nv_value())?;
        props.insert_u64(Copies::nv_key(), request.copies().as_nv_value())?;
        props.insert_u64("devices", bool_to_u64(request.devices))?;
        props.insert_u64("exec", bool_to_u64(request.exec))?;
        // saved fore mount point
        props.insert_u64("primarycache", request.primary_cache.as_nv_value())?;
        if let Some(quota) = request.quota {
            props.insert_u64("quota", quota)?;
        }
        props.insert_u64("readonly", bool_to_u64(request.readonly))?;
        if let Some(record_size) = request.record_size {
            props.insert_u64("recordsize", record_size)?;
        }
        if let Some(ref_quota) = request.ref_quota {
            props.insert_u64("refquota", ref_quota)?;
        }
        if let Some(ref_reservation) = request.ref_reservation {
            props.insert_u64("refreservation", ref_reservation)?;
        }
        props.insert_u64("secondarycache", request.secondary_cache.as_nv_value())?;
        props.insert_u64("setuid", bool_to_u64(request.setuid))?;
        props.insert_u64(SnapDir::nv_key(), request.snap_dir.as_nv_value())?;

        if request.kind == DatasetKind::Filesystem
            && (request.volume_size.is_some() || request.volume_block_size.is_some())
        {
            return Err(Error::invalid_input());
        }

        if request.kind == DatasetKind::Volume && request.volume_size.is_none() {
            return Err(Error::invalid_input());
        }

        if let Some(vol_size) = request.volume_size {
            props.insert_u64("volsize", vol_size)?;
        }
        if let Some(vol_block_size) = request.volume_block_size {
            props.insert_u64("volblocksize", vol_block_size)?;
        }

        props.insert("xattr", bool_to_u64(request.xattr))?;
        if let Some(user_props) = request.user_properties() {
            for (key, value) in user_props {
                props.insert_string(key, value)?;
            }
        }
        let errno = unsafe {
            zfs_core_sys::lzc_create(
                name_c_string.as_ref().as_ptr(),
                request.kind().as_c_uint(),
                props.as_ptr(),
            )
        };

        match errno {
            0 => Ok(()),
            _ => {
                let io_error = std::io::Error::from_raw_os_error(errno);
                Err(Error::Io(io_error))
            },
        }
    }

    fn snapshot(
        &self,
        snapshots: &[PathBuf],
        user_properties: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let validation_errors: Vec<ValidationError> = snapshots
            .iter()
            .map(PathBuf::validate)
            .filter(Result::is_err)
            .map(Result::unwrap_err)
            .collect();
        if !validation_errors.is_empty() {
            return Err(ValidationErrors(validation_errors));
        }

        let mut snapshots_list = NvList::default();
        let mut props = NvList::default();
        for snap in snapshots {
            snapshots_list.insert(&snap.to_string_lossy(), true)?;
        }
        let mut errors_list_ptr = null_mut();
        if let Some(user_properties) = user_properties {
            for (key, value) in user_properties {
                props.insert_string(&key, &value)?;
            }
        }
        let errno = unsafe {
            zfs_core_sys::lzc_snapshot(
                snapshots_list.as_ptr(),
                props.as_ptr(),
                &mut errors_list_ptr,
            )
        };
        if !errors_list_ptr.is_null() {
            let errors = unsafe { NvList::from_ptr(errors_list_ptr) };
            if !errors.is_empty() {
                return Err(Error::from(errors));
            }
        }
        match errno {
            0 => Ok(()),
            _ => {
                let io_error = std::io::Error::from_raw_os_error(errno);
                Err(Error::Io(io_error))
            },
        }
    }

    fn destroy_snapshots(&self, snapshots: &[PathBuf], timing: DestroyTiming) -> Result<()> {
        let validation_errors: Vec<ValidationError> = snapshots
            .iter()
            .map(PathBuf::validate)
            .filter(Result::is_err)
            .map(Result::unwrap_err)
            .collect();
        if !validation_errors.is_empty() {
            return Err(ValidationErrors(validation_errors));
        }

        let mut snapshots_list = NvList::default();

        for snap in snapshots {
            snapshots_list.insert(&snap.to_string_lossy(), true)?;
        }

        let mut errors_list_ptr = null_mut();
        let errno = unsafe {
            zfs_core_sys::lzc_destroy_snaps(
                snapshots_list.as_ptr(),
                timing.as_c_uint(),
                &mut errors_list_ptr,
            )
        };
        if !errors_list_ptr.is_null() {
            let errors = unsafe { NvList::from_ptr(errors_list_ptr) };
            if !errors.is_empty() {
                return Err(Error::from(errors));
            }
        }
        match errno {
            0 => Ok(()),
            _ => {
                let io_error = std::io::Error::from_raw_os_error(errno);
                Err(Error::Io(io_error))
            },
        }
    }
}

// This should be mapped to values from nvpair.
fn bool_to_u64(src: bool) -> u64 {
    if src {
        0
    } else {
        1
    }
}
