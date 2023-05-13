//! Open3 implementation of [`ZpoolEngine`](trait.ZpoolEngine.html).
//!
//! Easy way - [`ZpoolOpen3::default()`](struct.ZpoolOpen3.html#impl-Default).
//! It will look for `ZPOOL_CMD` in current
//! environment and fall back to `zpool` in `PATH`.
//!
//! Another way to specify is to use `ZpoolOpen3::new("/path/to/my/zpool")`.
//!
//! ### Usage
//! ```rust,no_run
//! use libzetta::zpool::{ZpoolEngine, ZpoolOpen3};
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
//! It's called [open3](https://docs.ruby-lang.org/en/2.0.0/Open3.html) because it opens `stdin`, `stdout`, `stderr`.

use std::{
    env,
    ffi::{OsStr, OsString},
    path::PathBuf,
    process::{Command, Output, Stdio},
};

use crate::{
    parsers::{Rule, StdoutParser},
    zpool::description::Zpool,
    GlobalLogger,
};
use pest::Parser;
use slog::Logger;

use super::{
    CreateMode, CreateVdevRequest, CreateZpoolRequest, DestroyMode, ExportMode, OfflineMode,
    OnlineMode, PropPair, ZpoolEngine, ZpoolError, ZpoolProperties, ZpoolResult,
};

lazy_static! {
    static ref ZPOOL_PROP_ARG: OsString = {
        let mut arg = OsString::with_capacity(171);
        arg.push("alloc,cap,comment,dedupratio,expandsize,fragmentation,free,");
        arg.push("freeing,guid,health,size,leaked,altroot,readonly,autoexpand,");
        arg.push("autoreplace,bootfs,cachefile,dedupditto,delegation,failmode");
        arg
    };
}
/// Open3 implementation of [`ZpoolEngine`](../trait.ZpoolEngine.html). You can use
/// `ZpoolOpen3::default` to create it.
pub struct ZpoolOpen3 {
    cmd_name: OsString,
    logger: Logger,
}

impl Default for ZpoolOpen3 {
    /// Uses `log` crate as drain for `Slog`. Tries to use `ZPOOL_CMD` from environment if variable
    /// is missing then it uses `zpool` from `$PATH`.
    fn default() -> ZpoolOpen3 {
        let cmd_name = match env::var_os("ZPOOL_CMD") {
            Some(val) => val,
            None => "zpool".into(),
        };

        let logger =
            GlobalLogger::get().new(o!("zetta_module" => "zpool", "zpool_impl" => "open3"));
        ZpoolOpen3 { cmd_name, logger }
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

    fn zpool(&self) -> Command {
        Command::new(&self.cmd_name)
    }

    #[allow(dead_code)]
    /// Force disable logging by using `/dev/null` as drain.
    fn zpool_mute(&self) -> Command {
        let mut z = self.zpool();
        z.stdout(Stdio::null());
        z.stderr(Stdio::null());
        z
    }

    fn zpools_from_import(&self, out: Output) -> ZpoolResult<Vec<Zpool>> {
        if out.status.success() {
            let stdout: String = String::from_utf8_lossy(&out.stdout).into();
            StdoutParser::parse(Rule::zpools, stdout.as_ref())
                .map_err(|_| ZpoolError::ParseError)
                .map(|pairs| pairs.map(Zpool::from_pest_pair).collect())
        } else {
            if out.stderr.is_empty() && out.stdout.is_empty() {
                return Ok(Vec::new());
            }
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }
}

#[derive(Default, Builder, Debug, Clone, Getters)]
#[builder(setter(into))]
#[get = "pub"]
pub struct StatusOptions {
    #[builder(default)]
    full_paths: bool,
    #[builder(default)]
    resolve_links: bool,
}

impl ZpoolEngine for ZpoolOpen3 {
    fn exists<N: AsRef<str>>(&self, name: N) -> ZpoolResult<bool> {
        let mut z = self.zpool_mute();
        z.arg("list").arg(name.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let status = z.status()?;
        Ok(status.success())
    }

    fn create(&self, request: CreateZpoolRequest) -> ZpoolResult<()> {
        if !request.is_suitable_for_create() {
            return Err(ZpoolError::InvalidTopology);
        }
        let mut z = self.zpool();
        z.arg("create");
        if request.create_mode() == &CreateMode::Force {
            z.arg("-f");
        }
        if let Some(props) = request.props().clone() {
            for arg in props.into_args() {
                z.arg("-o");
                z.arg(arg);
            }
        }
        if let Some(mount) = request.mount().clone() {
            z.arg("-m");
            z.arg(mount);
        }
        if let Some(altroot) = request.altroot().clone() {
            z.arg("-R");
            z.arg(altroot);
        }
        z.arg(request.name());
        z.args(request.into_args());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }

    fn destroy<N: AsRef<str>>(&self, name: N, mode: DestroyMode) -> ZpoolResult<()> {
        let mut z = self.zpool_mute();
        z.arg("destroy");
        if let DestroyMode::Force = mode {
            z.arg("-f");
        }
        z.arg(name.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        z.status().map(|_| Ok(()))?
    }

    fn read_properties<N: AsRef<str>>(&self, name: N) -> ZpoolResult<ZpoolProperties> {
        let mut z = self.zpool();
        z.args(&["list", "-p", "-H", "-o"]);
        z.arg(&*ZPOOL_PROP_ARG);
        z.arg(name.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            ZpoolProperties::try_from_stdout(&out.stdout)
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }

    fn set_property<N: AsRef<str>, P: PropPair>(
        &self,
        name: N,
        key: &str,
        value: &P,
    ) -> ZpoolResult<()> {
        let mut z = self.zpool();
        z.arg("set");
        z.arg(OsString::from(PropPair::to_pair(value, key)));
        z.arg(name.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }

    fn export<N: AsRef<str>>(&self, name: N, mode: ExportMode) -> ZpoolResult<()> {
        let mut z = self.zpool();
        z.arg("export");
        if let ExportMode::Force = mode {
            z.arg("-f");
        }
        z.arg(name.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }

    fn available(&self) -> ZpoolResult<Vec<Zpool>> {
        let mut z = self.zpool();
        z.arg("import");
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        self.zpools_from_import(out)
    }

    fn available_in_dir(&self, dir: PathBuf) -> ZpoolResult<Vec<Zpool>> {
        let mut z = self.zpool();
        z.arg("import");
        z.arg("-d");
        z.arg(dir);
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        self.zpools_from_import(out)
    }

    fn import<N: AsRef<str>>(&self, name: N) -> Result<(), ZpoolError> {
        let mut z = self.zpool();
        z.arg("import");
        z.arg(name.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }

    fn import_from_dir<N: AsRef<str>>(&self, name: N, dir: PathBuf) -> ZpoolResult<()> {
        let mut z = self.zpool();
        z.arg("import");
        z.arg("-d");
        z.arg(dir);
        z.arg(name.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }

    fn status<N: AsRef<str>>(&self, name: N, opts: StatusOptions) -> ZpoolResult<Zpool> {
        let mut z = self.zpool();
        z.arg("status");
        if opts.full_paths {
            z.arg("-P");
        }
        if opts.resolve_links {
            z.arg("-L");
        }
        z.arg(name.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        let zpools = self
            .zpools_from_import(out)
            .expect("Failed to unwrap zpool from status check");
        if zpools.is_empty() {
            return Err(ZpoolError::PoolNotFound);
        }
        let zpool = zpools.into_iter().next().expect("Can't build zpool out of pair. Please report at: https://github.com/Inner-Heaven/libzetta-rs");
        if zpool.name().as_str() != name.as_ref() {
            unreachable!();
        }
        Ok(zpool)
    }

    fn status_all(&self, opts: StatusOptions) -> ZpoolResult<Vec<Zpool>> {
        let mut z = self.zpool();
        z.arg("status");
        if opts.full_paths {
            z.arg("-P");
        }
        if opts.resolve_links {
            z.arg("-L");
        }
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        self.zpools_from_import(out)
    }

    fn scrub<N: AsRef<str>>(&self, name: N) -> ZpoolResult<()> {
        let mut z = self.zpool();
        z.arg("scrub");
        z.arg(name.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }

    fn pause_scrub<N: AsRef<str>>(&self, name: N) -> ZpoolResult<()> {
        let mut z = self.zpool();
        z.arg("scrub");
        z.arg("-p");
        z.arg(name.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }

    fn stop_scrub<N: AsRef<str>>(&self, name: N) -> ZpoolResult<()> {
        let mut z = self.zpool();
        z.arg("scrub");
        z.arg("-s");
        z.arg(name.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }

    fn take_offline<N: AsRef<str>, D: AsRef<OsStr>>(
        &self,
        name: N,
        device: D,
        mode: OfflineMode,
    ) -> ZpoolResult<()> {
        let mut z = self.zpool();
        z.arg("offline");
        if mode == OfflineMode::UntilReboot {
            z.arg("-t");
        }
        z.arg(name.as_ref());
        z.arg(device.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }

    fn bring_online<N: AsRef<str>, D: AsRef<OsStr>>(
        &self,
        name: N,
        device: D,
        mode: OnlineMode,
    ) -> ZpoolResult<()> {
        let mut z = self.zpool();
        z.arg("online");
        if mode == OnlineMode::Expand {
            z.arg("-e");
        }
        z.arg(name.as_ref());
        z.arg(device.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }

    fn attach<N: AsRef<str>, D: AsRef<OsStr>>(
        &self,
        name: N,
        device: D,
        new_device: D,
    ) -> ZpoolResult<()> {
        let mut z = self.zpool();
        z.arg("attach");
        z.arg(name.as_ref());
        z.arg(device.as_ref());
        z.arg(new_device.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }

    fn detach<N: AsRef<str>, D: AsRef<OsStr>>(&self, name: N, device: D) -> ZpoolResult<()> {
        let mut z = self.zpool();
        z.arg("detach");
        z.arg(name.as_ref());
        z.arg(device.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }

    fn add_vdev<N: AsRef<str>>(
        &self,
        name: N,
        new_vdev: CreateVdevRequest,
        add_mode: CreateMode,
    ) -> Result<(), ZpoolError> {
        let mut z = self.zpool();
        z.arg("add");
        if add_mode == CreateMode::Force {
            z.arg("-f");
        }
        z.arg(name.as_ref());
        z.args(new_vdev.into_args());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }

    fn add_zil<N: AsRef<str>>(
        &self,
        name: N,
        new_zil: CreateVdevRequest,
        add_mode: CreateMode,
    ) -> Result<(), ZpoolError> {
        let mut z = self.zpool();
        z.arg("add");
        if add_mode == CreateMode::Force {
            z.arg("-f");
        }
        z.arg(name.as_ref());
        z.arg("log");
        z.args(new_zil.into_args());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }

    fn add_cache<N: AsRef<str>, D: AsRef<OsStr>>(
        &self,
        name: N,
        new_cache: D,
        add_mode: CreateMode,
    ) -> Result<(), ZpoolError> {
        let mut z = self.zpool();
        z.arg("add");
        if add_mode == CreateMode::Force {
            z.arg("-f");
        }
        z.arg(name.as_ref());
        z.arg("cache");
        z.arg(new_cache.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }

    fn add_spare<N: AsRef<str>, D: AsRef<OsStr>>(
        &self,
        name: N,
        new_spare: D,
        add_mode: CreateMode,
    ) -> Result<(), ZpoolError> {
        let mut z = self.zpool();
        z.arg("add");
        if add_mode == CreateMode::Force {
            z.arg("-f");
        }
        z.arg(name.as_ref());
        z.arg("spare");
        z.arg(new_spare.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }

    fn replace_disk<N: AsRef<str>, D: AsRef<OsStr>, O: AsRef<OsStr>>(
        &self,
        name: N,
        old_disk: D,
        new_disk: O,
    ) -> Result<(), ZpoolError> {
        let mut z = self.zpool();
        z.arg("replace");
        z.arg(name.as_ref());
        z.arg(old_disk.as_ref());
        z.arg(new_disk.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }

    fn remove<N: AsRef<str>, D: AsRef<OsStr>>(&self, name: N, device: D) -> ZpoolResult<()> {
        let mut z = self.zpool();
        z.arg("remove");
        z.arg(name.as_ref());
        z.arg(device.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }
}

#[cfg(test)]
mod test {
    use std::assert_eq;

    use super::*;

    #[test]
    fn correctly_parses_vdevs() {
        let stdout = include_str!("fixtures/status_with_block_device_nested");
        let zpools: Vec<Zpool> = StdoutParser::parse(Rule::zpools, stdout.as_ref())
            .map_err(|_| ZpoolError::ParseError)
            .map(|pairs| pairs.map(Zpool::from_pest_pair).collect())
            .unwrap();
        let drives = &zpools[0]
            .vdevs()
            .iter()
            .flat_map(|vdev| vdev.disks().iter())
            .map(|drive| drive.path().display().to_string())
            .collect::<Vec<String>>();

        let expected: Vec<String> = [
            "/dev/diskid/DISK-ZCT2K2R6",
            "/dev/diskid/DISK-ZCT2QVET",
            "/dev/diskid/DISK-WSD6B5L6",
            "/dev/diskid/DISK-ZCT2QWL9",
            "/dev/diskid/DISK-ZCT2QXEL",
            "/dev/diskid/DISK-ZCT2RH0W",
        ]
        .iter()
        .map(|d| d.to_string())
        .collect();
        assert_eq!(&expected, drives);
    }
}
