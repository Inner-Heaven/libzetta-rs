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
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use zpool::description::Zpool;
use parsers::{StdoutParser, Rule};
use pest::Parser;

lazy_static! {
    static ref ZPOOL_PROP_ARG: OsString = {
        let mut arg = OsString::with_capacity(171);
        arg.push("alloc,cap,comment,dedupratio,expandsize,fragmentation,free,");
        arg.push("freeing,guid,health,size,leaked,altroot,readonly,autoexpand,");
        arg.push("autoreplace,bootfs,cachefile,dedupditto,delegation,failmode");
        arg
    };
}
fn setup_logger<L: Into<Logger>>(logger: L) -> Logger {
    logger
        .into()
        .new(o!("module" => "zpool", "impl" => "open3", "version" => "0.1.0"))
}

use super::{
    PropPair, Topology, ZpoolEngine, ZpoolError, ZpoolProperties, ZpoolPropertiesWrite, ZpoolResult,
};
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
            cmd_name,
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

    #[allow(dead_code)]
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
                .map(|pairs| {
                    pairs.map(Zpool::from_pest_pair).collect()
                })
        } else {
            Err(ZpoolError::from_stderr(&out.stderr))
        }
    }
}

impl ZpoolEngine for ZpoolOpen3 {
    fn exists<N: AsRef<str>>(&self, name: N) -> ZpoolResult<bool> {
        let mut z = self.zpool_mute();
        z.arg("list").arg(name.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let status = z.status()?;
        Ok(status.success())
    }

    fn create_unchecked<
        N: AsRef<str>,
        P: Into<Option<ZpoolPropertiesWrite>>,
        M: Into<Option<PathBuf>>,
        A: Into<Option<PathBuf>>,
    >(
        &self,
        name: N,
        topology: Topology,
        props: P,
        mount: M,
        alt_root: A,
    ) -> ZpoolResult<()> {
        let mut z = self.zpool();
        z.arg("create");
        if let Some(props) = props.into() {
            for arg in props.into_args() {
                z.arg("-o");
                z.arg(arg);
            }
        }
        if let Some(mount) = mount.into() {
            z.arg("-m");
            z.arg(mount);
        }
        if let Some(alt_root) = alt_root.into() {
            z.arg("-R");
            z.arg(alt_root);
        }
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

    fn destroy_unchecked<N: AsRef<str>>(&self, name: N, force: bool) -> ZpoolResult<()> {
        let mut z = self.zpool_mute();
        z.arg("destroy");
        if force {
            z.arg("-f");
        }
        z.arg(name.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        z.status().map(|_| Ok(()))?
    }
    fn read_properties_unchecked<N: AsRef<str>>(&self, name: N) -> ZpoolResult<ZpoolProperties> {
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

    fn set_unchecked<N: AsRef<str>, P: PropPair>(
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
    fn export_unchecked<N: AsRef<str>>(&self, name: N, force: bool) -> ZpoolResult<()> {
        let mut z = self.zpool();
        z.arg("export");
        if force {
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
    /// Status of a single pool
    fn status_unchecked<N: AsRef<str>>(&self, name: N) -> ZpoolResult<Zpool> {
        let mut z = self.zpool();
        z.arg("status");
        z.arg(name.as_ref());
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        let zpools = self.zpools_from_import(out).expect("Failed to unwrap zpool from status check");
        if zpools.is_empty() {
            return Err(ZpoolError::PoolNotFound);
        }
        let zpool = zpools.into_iter().next().unwrap();
        if zpool.name().as_str() != name.as_ref() {
            error!(self.logger, "Somehow got wrong zpool?"; "wanted" => name.as_ref(), "got" => zpool.name().as_str());
            return Err(ZpoolError::PoolNotFound);
        }
        Ok(zpool)
    }

    /// Get a status of each pool active in the system
    fn all(&self) -> ZpoolResult<Vec<Zpool>> {
        let mut z = self.zpool();
        z.arg("status");
        debug!(self.logger, "executing"; "cmd" => format_args!("{:?}", z));
        let out = z.output()?;
        self.zpools_from_import(out)
    }
}
