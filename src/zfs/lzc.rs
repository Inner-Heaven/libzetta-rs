use crate::zfs::{Error, Result, ZfsEngine, CreateDatasetRequest, DatasetKind};
use cstr_argument::CStrArgument;
use slog::{Drain, Logger};
use slog_stdlog::StdLog;
use libnv::nvpair::{NvList,NvTypeOp};

use zfs_core_sys as sys;
use std::ffi::{CStr, CString};

fn setup_logger<L: Into<Logger>>(logger: L) -> Logger {
    logger
        .into()
        .new(o!("zetta_module" => "zfs", "zfs_impl" => "lzc", "zetta_version" => crate::VERSION))
}

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
}

impl ZfsEngine for ZfsLzc {
    fn exists<D: CStrArgument>(&self, name: D) -> Result<bool, Error> {
        let n = name.into_cstr();
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

        dbg!(&name_c_string);

        /*
        if let Some(user_props) = request.user_properties() {
            for (key, value) in user_props {
                insert_str_into_nv_list(&key, &value, &mut props)?;
            }
        }
        */
        //insert_str_into_nv_list("copies", &request.copies().as_ref(), &mut props)?;
        props.insert("copies", request.copies().as_u64());
        dbg!("inserted");
        //nvpair::NvEncode::insert(&request.copies().as_u32(), "copies", &mut props)?;
        let errno = unsafe {
            zfs_core_sys::lzc_create(name_c_string.as_ref().as_ptr(), request.kind().as_c_uint(), props.as_ptr())
        };

        match errno {
            0  => Ok(()),
            22 => Err(Error::InvalidInput),
            _  => {

                let io_error = std::io::Error::from_raw_os_error(errno);
                Err(Error::Io(io_error))
            }
        }
    }
}
