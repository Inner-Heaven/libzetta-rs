use crate::zfs::{DatasetKind, Error, FilesystemProperties, Properties, Result, ZfsEngine};
use chrono::NaiveDateTime;
use slog::{Drain, Logger};
use slog_stdlog::StdLog;
use std::{ffi::OsString,
          path::PathBuf,
          process::{Command, Stdio}};

use crate::{parsers::zfs::{Rule, ZfsParser},
            utils::parse_float,
            zfs::properties::SnapshotProperties};
use pest::Parser;
use std::str::Lines;

static FAILED_TO_PARSE: &str = "Failed to parse value";
static DATE_FORMAT: &str = "%a %b %e %k:%M %Y";

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
            let kind = parse_prop_line(&first).1;
            let ret = match kind.as_ref() {
                "filesystem" => parse_filesystem_lines(&mut lines),
                "snapshot" => parse_snapshot_lines(&mut lines),
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

fn parse_list_of_pathbufs(value: &str) -> Option<Vec<PathBuf>> {
    if value == "-" || value == "" {
        return None;
    }
    let clones = value.split(",").map(PathBuf::from).collect();
    Some(clones)
}

fn parse_creation_into_timestamp(value: &str) -> i64 {
    if let Ok(timestamp) = value.parse() {
        return timestamp;
    } else {
        let date = NaiveDateTime::parse_from_str(value, DATE_FORMAT).expect(FAILED_TO_PARSE);
        return date.timestamp();
    };
}

pub(crate) fn parse_filesystem_lines(lines: &mut Lines) -> Properties {
    let mut properties = FilesystemProperties::builder();
    for (key, value) in lines.map(parse_prop_line) {
        match key.as_ref() {
            "aclinherit" => {
                properties.acl_inherit(value.parse().expect(FAILED_TO_PARSE));
            },
            "aclmode" => {
                properties.acl_mode(Some(value.parse().expect(FAILED_TO_PARSE)));
            },
            "atime" => {
                properties.atime(parse_bool(&value));
            },
            "available" => {
                properties.available(value.parse().expect(FAILED_TO_PARSE));
            },
            "canmount" => {
                properties.can_mount(value.parse().expect(FAILED_TO_PARSE));
            },
            "checksum" => {
                properties.checksum(value.parse().expect(FAILED_TO_PARSE));
            },
            "compression" => {
                properties.compression(value.parse().expect(FAILED_TO_PARSE));
            },
            "compressratio" => {
                properties
                    .compression_ratio(parse_float(&mut value.clone()).expect(FAILED_TO_PARSE));
            },
            "copies" => {
                properties.copies(value.parse().expect(FAILED_TO_PARSE));
            },
            "creation" => {
                properties.creation(value.parse().expect(FAILED_TO_PARSE));
            },
            "devices" => {
                properties.devices(parse_bool(&value));
            },
            "exec" => {
                properties.exec(parse_bool(&value));
            },
            "guid" => {
                properties.guid(Some(value.parse().expect(FAILED_TO_PARSE)));
            },
            "jailed" => {
                properties.jailed(Some(parse_bool(&value)));
            },
            "mounted" => {
                properties.mounted(parse_bool(&value));
            },
            "mountpoint" => {
                properties.mount_point(parse_mount_point(&value));
            },
            "origin" => {
                properties.origin(Some(value));
            },
            "primarycache" => {
                properties.primary_cache(value.parse().expect(FAILED_TO_PARSE));
            },
            "quota" => {
                properties.quota(value.parse().expect(FAILED_TO_PARSE));
            },
            "readonly" => {
                properties.readonly(parse_bool(&value));
            },
            "recordsize" => {
                properties.record_size(value.parse().expect(FAILED_TO_PARSE));
            },
            "refquota" => {
                properties.ref_quota(value.parse().expect(FAILED_TO_PARSE));
            },
            "refreservation" => {
                properties.ref_reservation(value.parse().expect(FAILED_TO_PARSE));
            },
            "referenced" => {
                properties.referenced(value.parse().expect(FAILED_TO_PARSE));
            },
            "reservation" => {
                properties.reservation(value.parse().expect(FAILED_TO_PARSE));
            },
            "secondarycache" => {
                properties.secondary_cache(value.parse().expect(FAILED_TO_PARSE));
            },
            "setuid" => {
                properties.setuid(parse_bool(&value));
            },
            "snapdir" => {
                properties.snap_dir(value.parse().expect(FAILED_TO_PARSE));
            },
            "sync" => {
                properties.sync(value.parse().expect(FAILED_TO_PARSE));
            },
            "used" => {
                properties.used(value.parse().expect(FAILED_TO_PARSE));
            },
            "usedbychildren" => {
                properties.used_by_children(value.parse().expect(FAILED_TO_PARSE));
            },
            "usedbydataset" => {
                properties.used_by_dataset(value.parse().expect(FAILED_TO_PARSE));
            },
            "usedbyrefreservation" => {
                properties.used_by_ref_reservation(value.parse().expect(FAILED_TO_PARSE));
            },
            "usedbysnapshots" => {
                properties.used_by_snapshots(value.parse().expect(FAILED_TO_PARSE));
            },
            "utf8only" => {
                properties.utf8_only(Some(parse_bool(&value)));
            },
            "version" => {
                properties.version(value.parse().expect(FAILED_TO_PARSE));
            },
            "volmode" => {
                properties.volume_mode(Some(value.parse().expect(FAILED_TO_PARSE)));
            },
            "written" => {
                properties.written(value.parse().expect(FAILED_TO_PARSE));
            },
            "xattr" => {
                properties.xattr(parse_bool(&value));
            },
            "type" => { /* no-op */ },

            _ => properties.insert_unknown_property(key, value),
        };
    }
    Properties::Filesystem(properties.build().expect("Failed to build properties"))
}

pub(crate) fn parse_snapshot_lines(lines: &mut Lines) -> Properties {
    let mut properties = SnapshotProperties::builder();
    for (key, value) in lines.map(parse_prop_line) {
        match key.as_ref() {
            "clones" => {
                properties.clones(parse_list_of_pathbufs(&value));
            },
            "compressratio" => {
                properties
                    .compression_ratio(parse_float(&mut value.clone()).expect(FAILED_TO_PARSE));
            },
            "creation" => {
                properties.creation(parse_creation_into_timestamp(&value));
            },
            "defer_destroy" => {
                properties.defer_destroy(parse_bool(&value));
            },
            "devices" => {
                properties.devices(parse_bool(&value));
            },
            "exec" => {
                properties.exec(parse_bool(&value));
            },
            "guid" => {
                properties.guid(Some(value.parse().expect(FAILED_TO_PARSE)));
            },
            "logicalreferenced" => {
                properties.logically_referenced(value.parse().expect(FAILED_TO_PARSE));
            },
            "primarycache" => {
                properties.primary_cache(value.parse().expect(FAILED_TO_PARSE));
            },
            "refcompressratio" => {
                properties
                    .ref_compression_ratio(parse_float(&mut value.clone()).expect(FAILED_TO_PARSE));
            },
            "referenced" => {
                properties.referenced(value.parse().expect(FAILED_TO_PARSE));
            },
            "secondarycache" => {
                properties.secondary_cache(value.parse().expect(FAILED_TO_PARSE));
            },
            "setuid" => {
                properties.setuid(parse_bool(&value));
            },
            "used" => {
                properties.used(value.parse().expect(FAILED_TO_PARSE));
            },
            "userrefs" => {
                properties.user_refs(value.parse().expect(FAILED_TO_PARSE));
            },
            "utf8only" => {
                properties.utf8_only(Some(parse_bool(&value)));
            },
            "version" => {
                properties.version(value.parse().expect(FAILED_TO_PARSE));
            },
            "volmode" => {
                properties.volume_mode(Some(value.parse().expect(FAILED_TO_PARSE)));
            },
            "written" => {
                properties.written(value.parse().expect(FAILED_TO_PARSE));
            },
            "xattr" => {
                properties.xattr(parse_bool(&value));
            },
            "type" => { /* no-op */ },

            _ => properties.insert_unknown_property(key, value),
        };
    }
    Properties::Snapshot(properties.build().expect("Failed to build properties"))
}
fn parse_unknown_lines(lines: &mut Lines) -> Properties {
    let props = lines.map(parse_prop_line).collect();
    Properties::Unknown(props)
}

fn parse_bool(val: &str) -> bool { val == "yes" || val == "on" }

fn parse_mount_point(val: &str) -> Option<PathBuf> {
    match val {
        "-" | "none" => None,
        _ => Some(PathBuf::from(val)),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::zfs::{properties::{AclInheritMode, AclMode, SnapshotProperties, SyncMode,
                                  VolumeMode},
                     CacheMode, CanMount, Checksum, Compression, Copies, SnapDir};
    use std::collections::HashMap;

    #[test]
    fn test_hashmap_eq() {
        let mut left = HashMap::new();
        left.insert("foo", "bar");
        left.insert("bar", "foo");
        let mut right = HashMap::new();
        right.insert("bar", "foo");
        right.insert("foo", "bar");
        assert_eq!(left, right);
    }
    #[test]
    fn filesystem_properties_freebsd() {
        let stdout = include_str!("fixtures/filesystem_properties_freebsd.sorted");

        let result = parse_filesystem_lines(&mut stdout.lines());

        // Goal to have zero unknown before 1.0
        let unknown = [
            ("casesensitivity", "sensitive"),
            ("createtxg", "46918"),
            ("dedup", "off"),
            ("dnodesize", "legacy"),
            ("filesystem_count", "18446744073709551615"),
            ("filesystem_limit", "18446744073709551615"),
            ("logbias", "latency"),
            ("logicalreferenced", "117966950912"),
            ("logicalused", "125882283520"),
            ("mlslabel", ""),
            ("nbmand", "off"),
            ("normalization", "none"),
            ("redundant_metadata", "all"),
            ("refcompressratio", "1.23x"),
            ("sharenfs", "off"),
            ("sharesmb", "off"),
            ("snapshot_count", "18446744073709551615"),
            ("snapshot_limit", "18446744073709551615"),
            ("vscan", "off"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

        let expected = FilesystemProperties::builder()
            .acl_inherit(AclInheritMode::Restricted)
            .acl_mode(Some(AclMode::Discard))
            .atime(false)
            .available(161379753984)
            .can_mount(CanMount::On)
            .checksum(Checksum::On)
            .compression(Compression::LZ4)
            .compression_ratio(1.25)
            .copies(Copies::One)
            .creation(1493670099)
            .devices(true)
            .exec(true)
            .guid(Some(10533576440524459469))
            .jailed(Some(false))
            .mounted(true)
            .mount_point(Some(PathBuf::from("/usr/home")))
            .primary_cache(CacheMode::All)
            .quota(0)
            .readonly(false)
            .record_size(131072)
            .referenced(97392148480)
            .ref_quota(0)
            .ref_reservation(0)
            .reservation(0)
            .secondary_cache(CacheMode::All)
            .setuid(true)
            .snap_dir(SnapDir::Hidden)
            .sync(SyncMode::Standard)
            .used(102563762176)
            .used_by_children(0)
            .used_by_dataset(97392148480)
            .used_by_ref_reservation(0)
            .used_by_snapshots(5171613696)
            .utf8_only(Some(false))
            .version(5)
            .written(35372666880)
            .xattr(false)
            .volume_mode(Some(VolumeMode::Default))
            .unknown_properties(unknown)
            .build()
            .unwrap();

        assert_eq!(Properties::Filesystem(expected), result);
    }

    #[test]
    fn snapshot_properties_freebsd() {
        let stdout = include_str!("fixtures/snapshot_properties_freebsd.sorted");

        let result = parse_snapshot_lines(&mut stdout.lines());

        // Goal to have zero unknown before 1.0
        let unknown = [
            ("casesensitivity", "sensitive"),
            ("mlslabel", ""),
            ("nbmand", "off"),
            ("normalization", "none"),
            ("createtxg", "3034392"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

        let expected = SnapshotProperties::builder()
            .clones(None)
            .compression_ratio(1.0)
            .creation(1574590597)
            .defer_destroy(false)
            .devices(true)
            .exec(true)
            .guid(Some(6033436932844487115))
            .logically_referenced(37376)
            .primary_cache(CacheMode::All)
            .ref_compression_ratio(1.0)
            .referenced(90210)
            .secondary_cache(CacheMode::All)
            .setuid(true)
            .used(0)
            .user_refs(0)
            .utf8_only(Some(false))
            .version(5)
            .volume_mode(Some(VolumeMode::Default))
            .written(0)
            .xattr(true)
            .unknown_properties(unknown)
            .build()
            .unwrap();

        assert_eq!(Properties::Snapshot(expected), result);
    }
}
