use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to initialize libzfs_core. Error: {}", errno))]
    ZFSInitializationFailed{ errno: std::os::raw::c_int },
}

type Result<T, E = Error> = std::result<T,E>;
