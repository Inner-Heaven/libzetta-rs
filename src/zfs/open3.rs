use crate::zfs::{CreateDatasetRequest, Error, Result, ZfsEngine};
use cstr_argument::CStrArgument;
use slog::{Drain, Logger};
use slog_stdlog::StdLog;
use std::{ffi::OsString,
          path::PathBuf,
          process::{Command, Stdio}};

fn setup_logger<L: Into<Logger>>(logger: L) -> Logger {
    logger
        .into()
        .new(o!("zetta_module" => "zfs", "zfs_impl" => "open3", "zetta_version" => crate::VERSION))
}

pub struct ZfsOpen3 {
    cmd_name: OsString,
    logger:   Logger,
}

impl ZfsOpen3 {
    /// Initialize libzfs_core backed ZfsEngine.
    /// If root logger is None, then StdLog drain used.
    pub fn new(root_logger: Option<Logger>) -> Self {
        let logger = {
            if let Some(slog) = root_logger {
                setup_logger(slog)
            } else {
                let slog = Logger::root(StdLog.fuse(), o!());
                setup_logger(slog)
            }
        };
        let cmd_name = match std::env::var_os("ZFS_CMD") {
            Some(val) => val,
            None => "zfs".into(),
        };

        ZfsOpen3 { logger, cmd_name }
    }

    pub fn logger(&self) -> &Logger { &self.logger }

    fn zfs(&self) -> Command { Command::new(&self.cmd_name) }

    #[allow(dead_code)]
    /// Force disable logging by using `/dev/null` as drain.
    fn zfs_mute(&self) -> Command {
        let mut z = self.zfs();
        z.stdout(Stdio::null());
        z.stderr(Stdio::null());
        z
    }
}

impl ZfsEngine for ZfsOpen3 {
    fn exists<N: Into<PathBuf>>(&self, _name: N) -> Result<bool> { unimplemented!() }

    fn create(&self, _request: CreateDatasetRequest) -> Result<()> { unimplemented!() }

    fn destroy<N: Into<PathBuf>>(&self, name: N) -> Result<()> {
        let mut z = self.zfs_mute();
        z.arg("destroy");
        z.arg(name.into().as_os_str());

        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(Error::Unknown)
        }
    }
}
