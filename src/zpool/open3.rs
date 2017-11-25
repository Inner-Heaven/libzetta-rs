//! Open3 implementation of [`ZpoolEngine`](trait.ZpoolEngine.html).
//!
//! Easy way - [`ZpoolOpen3::default()`](struct.ZpoolOpen3.html#impl-Default). It will look for `ZPOOL_CMD` in current
//! environment and fall back to `zpool` in `PATH`.
//!
//! Another way to specify is to use `ZpoolOpen3::new("/path/to/my/zpool")`.
//!
//! ### Usage
//! ```rust,no_run
//! use libzfs::zpool::{ZpoolEngine, ZpoolOpen3};
//! let engine = ZpoolOpen3::default();
//!
//! // Check that pool with name z exists.
//! assert!(engine.exists("z").unwrap());
//!
//! let remote = ZpoolOpen3::with_cmd("zpool.sh");
//!
//! assert!(engine.exists("z").unwrap());
//! ```
//!
//! It's called open 3 because it opens stdin, stdout, stder.
use slog::{Drain, Logger};
use slog_stdlog::StdLog;
use std::env;
use std::ffi::OsString;
use std::process::{Command, Stdio};

use std::{thread, time};
fn setup_logger<L: Into<Logger>>(logger: L) -> Logger {
    logger.into()
          .new(o!("module" => "zpool", "impl" => "open3", "version" => "0.1.0"))
}

use super::{Topology, ZpoolEngine, ZpoolResult, ZpoolError};
pub struct ZpoolOpen3 {
    cmd_name: OsString,
    logger: Logger,
}

impl Default for ZpoolOpen3 {
    fn default() -> ZpoolOpen3 {
        let cmd_name = match env::var_os("ZPOOL_CMD") {
            Some(val) => val,
            None => "zpool".into(),
        };

        let logger = Logger::root(StdLog.fuse(), o!());
        ZpoolOpen3 {
            cmd_name: cmd_name,
            logger: setup_logger(logger),
        }
    }
}
impl ZpoolOpen3 {
    /// Create new using supplied path as zpool cmd using "log" as backend for
    /// logging.
    pub fn with_cmd<I: Into<OsString>>(cmd_name: I) -> ZpoolOpen3 {
        let mut z = ZpoolOpen3::default();
        z.cmd_name = cmd_name.into();
        z
    }

    /// Create new using supplies logger and default cmd.
    pub fn with_logger<L: Into<Logger>>(logger: L) -> ZpoolOpen3 {
        let mut z = ZpoolOpen3::default();
        z.logger = setup_logger(logger);
        z
    }

    fn zpool(&self) -> Command { Command::new(&self.cmd_name) }

    fn zpool_mute(&self) -> Command {
        let mut z = self.zpool();
        z.stdout(Stdio::null());
        z.stderr(Stdio::null());
        z
    }
}

impl ZpoolEngine for ZpoolOpen3 {
    fn exists<N: AsRef<str>>(&self, name: N) -> ZpoolResult<bool> {
        let mut z = self.zpool();
        z.arg("list").arg(name.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let status = z.status()?;
        Ok(status.success())
    }

    fn destroy<N: AsRef<str>>(&self, name: N, force: bool) -> ZpoolResult<()> {
        Command::new("fuser")
            .arg("-c")
            .arg("/tests")
            .status()
            .unwrap();
        let mut z = self.zpool();
        z.arg("destroy");
        if force {
            z.arg("-f");
        }
        z.arg(name.as_ref());
        let ten_millis = time::Duration::from_secs(5);
        thread::sleep(ten_millis);
        Command::new("fuser")
            .arg("-c")
            .arg("/tests")
            .status()
            .unwrap();
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        z.status().map(|_| Ok(()))?
    }

    fn create<N: AsRef<str>>(&self, name: N, topology: Topology) -> ZpoolResult<()> {
        let mut z = self.zpool();
        z.arg("create");
        z.arg(name.as_ref());
        z.args(topology.into_args());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }
}
