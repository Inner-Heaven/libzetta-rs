use crate::zfs::{DatasetKind, Error, Result, ZfsEngine};
use slog::{Drain, Logger};
use slog_stdlog::StdLog;
use std::{ffi::OsString,
          path::PathBuf,
          process::{Command, Stdio}};

use crate::parsers::zfs::{Rule, ZfsParser};
use pest::Parser;

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

    fn list<N: Into<PathBuf>>(&self, prefix: N) -> Result<Vec<(DatasetKind, PathBuf)>> {
        let mut z = self.zfs();
        z.args(&["list", "-t", "all", "-o", "type,name", "-Hpr"]);
        z.arg(prefix.into().as_os_str());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));

        let out = z.output()?;
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            ZfsParser::parse(Rule::datasets_with_type, &stdout)
                .map(|mut pairs| {
                    pairs
                        .next()
                        .unwrap()
                        .into_inner()
                        .map(|pair| {
                            //
                            // - datasets_with_type
                            //   - dataset_with_type
                            //     - dataset_type: "volume"
                            //     - dataset_name: "z/iohyve/rancher/disk0"
                            debug_assert_eq!(Rule::dataset_with_type, pair.as_rule());
                            let mut inner = pair.into_inner();

                            let dataset_type_pair = inner.next().unwrap();
                            let dataset_name_pair = inner.next().unwrap();
                            let dataset_type = dataset_type_pair.as_str().parse().unwrap();
                            let dataset_name = PathBuf::from(dataset_name_pair.as_str());
                            (dataset_type, dataset_name)
                        })
                        .collect()
                })
                .map_err(|_| Error::UnknownSoFar(String::from(stdout)))
        } else {
            Err(Error::from_stderr(&out.stderr))
        }
    }

    fn list_filesystems<N: Into<PathBuf>>(&self, pool: N) -> Result<Vec<PathBuf>> {
        let mut z = self.zfs();
        z.args(&["list", "-t", "filesystem", "-o", "name", "-Hpr"]);
        z.arg(pool.into().as_os_str());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        ZfsOpen3::stdout_to_list_of_datasets(&mut z)
    }

    fn list_snapshots<N: Into<PathBuf>>(&self, pool: N) -> Result<Vec<PathBuf>> {
        let mut z = self.zfs();
        z.args(&["list", "-t", "snapshot", "-o", "name", "-Hpr"]);
        z.arg(pool.into().as_os_str());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        ZfsOpen3::stdout_to_list_of_datasets(&mut z)
    }

    fn list_volumes<N: Into<PathBuf>>(&self, pool: N) -> Result<Vec<PathBuf>> {
        let mut z = self.zfs();
        z.args(&["list", "-t", "volume", "-o", "name", "-Hpr"]);
        z.arg(pool.into().as_os_str());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        ZfsOpen3::stdout_to_list_of_datasets(&mut z)
    }
}

impl ZfsOpen3 {
    fn stdout_to_list_of_datasets(z: &mut Command) -> Result<Vec<PathBuf>, Error> {
        let out = z.output()?;
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            ZfsParser::parse(Rule::datasets, &stdout)
                .map(|mut pairs| {
                    pairs
                        .next()
                        .unwrap()
                        .into_inner()
                        .map(|pair| {
                            debug_assert_eq!(Rule::dataset_name, pair.as_rule());
                            PathBuf::from(pair.as_str())
                        })
                        .collect()
                })
                .map_err(|_| Error::UnknownSoFar(String::from(stdout)))
        } else {
            Err(Error::from_stderr(&out.stderr))
        }
    }
}
