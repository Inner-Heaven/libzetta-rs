/// Open3 implementation of ZpoolEngine.
///
/// Easy way - `ZpooOpen3::default()`. It will look for `ZPOOL_CMD' in current environment and fall
/// back to `zpool` in `PATH`.
///
/// Another way to specify is to use `ZpoolOpen3::new("/path/to/my/zpool")`.
///
/// ### Usage
/// ```rust,no_run
/// use libzfs::zpool::{ZpoolEngine, ZpoolOpen3};
/// let engine = ZpoolOpen3::default();
///
/// // Check that pool with name z exists.
/// assert!(engine.exists("z").unwrap());
///
/// // Manage remote zpool
/// // where zpool.sh is a script that passes everything to `ssh user1@server1 zpool`
/// let remote = ZpoolOpen3::with_cmd("zpool.sh");
///
/// assert!(engine.exists("z").unwrap());
/// ```
///
/// It's called open 3 because it opens stdin, stdout, stder.
use std::process::{Command, Stdio};
use std::env;
use std::ffi::OsString;

use super::{ZpoolEngine, ZpoolResult, Topology, ZpoolProperties, ZpoolProperty};
pub struct ZpoolOpen3 {
    cmd_name: OsString
}

impl Default for ZpoolOpen3 {
    fn default() -> ZpoolOpen3 {
        let cmd_name = match env::var_os("ZPOOL_CMD") {
            Some(val)   => val,
            None        => "zpool".into()
        };

        ZpoolOpen3 {
            cmd_name: cmd_name
        }
    }
}
impl ZpoolOpen3 {
    /// Create new using supplied path as zpool zmd
    pub fn with_cmd<I: Into<OsString>>(cmd_name: I) -> ZpoolOpen3 {
        ZpoolOpen3 {
            cmd_name: cmd_name.into()
        }
    }

    fn zpool(&self) -> Command {
        Command::new(&self.cmd_name)
    }

    fn zpool_mute(&self) -> Command {
        let mut z = self.zpool();
        z.stdout(Stdio::null());
        z.stderr(Stdio::null());
        z
    }
}

impl ZpoolEngine for ZpoolOpen3 {
    fn exists<N: AsRef<str>>(&self, name: N) -> ZpoolResult<bool> {
        let mut z = self.zpool_mute();
        let cmd = z.arg("list").arg(name.as_ref());
        let status = cmd.status()?;
        Ok(status.success())
    }

    fn create<N: AsRef<str>>(&self, name: N, topology: Topology, properties: ZpoolProperties) -> ZpoolResult<()> {
        unimplemented!()
    }
}
