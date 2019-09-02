use crate::zfs::{Checksum, Compression, Copies, CreateDatasetRequest, DatasetKind, Error, Result,
                 SnapDir, ZfsEngine};
use cstr_argument::CStrArgument;
use libnv::nvpair::NvList;
use slog::{Drain, Logger};
use slog_stdlog::StdLog;

use crate::zfs::properties::{AclInheritMode, AclMode, ZfsProp};
use std::{ffi::CString, path::PathBuf};
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

    fn create(&self, request: CreateDatasetRequest) -> Result<(), Error> {
        //let mut props = nvpair::NvList::new()?;
        let mut props = NvList::default();
        let name_c_string = CString::new(request.name().to_str().unwrap()).unwrap();
        // LZC wants _everything_ as u64 even booleans.
        props.insert_u64(AclInheritMode::as_nv_key(), request.acl_inherit.as_nv_value())?;
        if let Some(acl_mode) = request.acl_mode {
            props.insert_u64(AclMode::as_nv_key(), acl_mode.as_nv_value())?;
        }
        props.insert_u64("atime", bool_to_u64(request.atime))?;
        props.insert_u64(Checksum::as_nv_key(), request.checksum.as_nv_value())?;
        props.insert_u64(Compression::as_nv_key(), request.compression.as_nv_value())?;
        props.insert_u64(Copies::as_nv_key(), request.copies().as_nv_value())?;
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
        props.insert_u64(SnapDir::as_nv_key(), request.snap_dir.as_nv_value())?;

        if request.kind == DatasetKind::Filesystem
            && (request.volume_size.is_some() || request.volume_block_size.is_some())
        {
            return Err(Error::InvalidInput);
        }

        if request.kind == DatasetKind::Volume && request.volume_size.is_none() {
            return Err(Error::InvalidInput);
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
                props.insert_string(&key, &value)?;
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
            22 => Err(Error::InvalidInput),
            _ => {
                let io_error = std::io::Error::from_raw_os_error(errno);
                Err(Error::Io(io_error))
            },
        }
    }

    fn destroy<N: Into<PathBuf>>(&self, _name: N) -> Result<(), Error> { unimplemented!() }
}

// This should be mapped to values from nvpair.
fn bool_to_u64(src: bool) -> u64 {
    if src {
        0
    } else {
        1
    }
}
