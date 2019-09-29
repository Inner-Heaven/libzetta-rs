use crate::zfs::{DatasetKind, Error, Result, ZfsEngine, Properties, FilesystemProperties};
use slog::{Drain, Logger};
use slog_stdlog::StdLog;
use std::{ffi::OsString,
          path::PathBuf,
          process::{Command, Stdio}};

use crate::parsers::zfs::{Rule, ZfsParser};
use pest::Parser;
use std::str::Lines;

static FAILED_TO_PARSE: &str = "Failed to parse value";

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

    #[allow(clippy::option_unwrap_used)]
    #[allow(clippy::result_unwrap_used)]
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

    fn read_properties<N: Into<PathBuf>>(&self, path: N) -> Result<Properties> {
        let mut z = self.zfs();
        z.args(&["get", "-Hp", "all"]);
        z.arg(path.into().as_os_str());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut lines = stdout.lines();

            let first = lines.next().expect("Empty stdout with 0 exit code");
            let kind = parse_prop_line(&first).0;

            let ret =match kind.as_ref() {
                "filesystem" => parse_filesystem_lines(&mut lines),
                _ => parse_unknown_lines(&mut lines),
            };
            Ok(ret)
        } else {
            Err(Error::from_stderr(&out.stderr))
        }
    }
}


impl ZfsOpen3 {
    #[allow(clippy::option_unwrap_used)]
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

fn parse_prop_line(line: &str) -> (String, String) {
    let mut splits = line.split('\t');
    // consume dataset name
    splits.next().expect("Failed to parse output");
    let name = splits.next().expect("failed to extract key").to_string();
    let value = splits.next().expect("Failed to extract value").to_string();
    (name, value)
}

pub(crate) fn parse_filesystem_lines(lines: &mut Lines) -> Properties {
    let mut properties = FilesystemProperties::builder();
    for (key, value) in lines.map(parse_prop_line) {
        match key.as_ref() {
            "creation" => { properties.creation(value.parse().expect(FAILED_TO_PARSE)); },
            "used" => { properties.used(value.parse().expect(FAILED_TO_PARSE)); },
            "available" => { properties.available(value.parse().expect(FAILED_TO_PARSE)); },
            "referenced" => { properties.referenced(value.parse().expect(FAILED_TO_PARSE)); },
            "compressratio" => { properties.compression_ratio(value.parse().expect(FAILED_TO_PARSE)); },
            "mounted" => { properties.mounted(parse_bool(&value)); },
            "quota"  => { properties.quota(value.parse().expect(FAILED_TO_PARSE)); },
            "reservation"  => { properties.reservation(value.parse().expect(FAILED_TO_PARSE)); },
            "recordsize"  => { properties.record_size(value.parse().expect(FAILED_TO_PARSE)); },
            "mountpoint" => { properties.mount_point(parse_mount_point(&value)); },

            _ => properties.insert_unknown_property(key, value),
        };
    }
    unimplemented!();
}

fn parse_unknown_lines(lines: &mut Lines) -> Properties {
    let props = lines.map(parse_prop_line).collect();
    Properties::Unknown(props)
}

fn parse_bool(val: &str) -> bool {
    val == "yes"
}

fn parse_mount_point(val: &str) -> Option<PathBuf> {
    match val {
        "-" | "none" => None,
        _ => Some(PathBuf::from(val))
    }
}