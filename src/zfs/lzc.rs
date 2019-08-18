use crate::zfs::{Error, Result, ZfsEngine};
use cstr_argument::CStrArgument;
use slog::{Drain, Logger};
use slog_stdlog::StdLog;

use zfs_core_sys as sys;

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
            return Err(Error::ZFSInitializationFailed(io_error));
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
}
