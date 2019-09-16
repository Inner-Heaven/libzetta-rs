use crate::parsers::zfs::{Rule, ZfsParser};
use pest::Parser;
use std::{borrow::Cow, io, path::PathBuf};

pub type Result<T, E = Error> = std::result::Result<T, E>;
pub type ValidationResult<T = (), E = ValidationError> = std::result::Result<T, E>;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        /// `zfs not found in the PATH. Open3 specific error.
        CmdNotFound {}
        LZCInitializationFailed(err: std::io::Error) {
            cause(err)
        }
        NvOpError(err: libnv::NvError) {
            cause(err)
            from()
        }
        InvalidInput {}
        Io(err: std::io::Error) {
            cause(err)
        }
        Unknown {}
        UnknownSoFar(err: String) {}
        DatasetNotFound(dataset: PathBuf) {}
        ValidationErrors(errors: Vec<ValidationError>) {
            from()
        }
    }
}

impl From<io::Error> for Error {
    #[allow(clippy::wildcard_enum_match_arm)]
    fn from(err: io::Error) -> Error {
        match err.kind() {
            io::ErrorKind::NotFound => Error::CmdNotFound,
            io::ErrorKind::InvalidInput => Error::InvalidInput,
            _ => Error::Io(err),
        }
    }
}
impl From<ValidationError> for Error {
    fn from(err: ValidationError) -> Error { Error::ValidationErrors(vec![err]) }
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        match self {
            Error::CmdNotFound => ErrorKind::CmdNotFound,
            Error::LZCInitializationFailed(_) => ErrorKind::LZCInitializationFailed,
            Error::NvOpError(_) => ErrorKind::NvOpError,
            Error::InvalidInput => ErrorKind::InvalidInput,
            Error::Io(_) => ErrorKind::Io,
            Error::DatasetNotFound(_) => ErrorKind::DatasetNotFound,
            Error::Unknown | Error::UnknownSoFar(_) => ErrorKind::Unknown,
            Error::ValidationErrors(_) => ErrorKind::ValidationErrors,
        }
    }

    fn unknown_so_far(stderr: Cow<'_, str>) -> Self { Error::UnknownSoFar(stderr.into()) }

    pub(crate) fn from_stderr(stderr_raw: &[u8]) -> Self {
        let stderr = String::from_utf8_lossy(stderr_raw);
        if let Ok(mut pairs) = ZfsParser::parse(Rule::error, &stderr) {
            // Pest: error > dataset_not_found > dataset_name: "s/asd/asd"
            let error_pair = pairs.next().unwrap().into_inner().next().unwrap();
            match error_pair.as_rule() {
                Rule::dataset_not_found => {
                    let dataset_name_pair = error_pair.into_inner().next().unwrap();
                    return Error::DatasetNotFound(PathBuf::from(dataset_name_pair.as_str()));
                },
                _ => return Self::unknown_so_far(stderr),
            }
        } else {
            Self::unknown_so_far(stderr)
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ErrorKind {
    CmdNotFound,
    LZCInitializationFailed,
    NvOpError,
    InvalidInput,
    Io,
    Unknown,
    DatasetNotFound,
    ValidationErrors,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Error::ValidationErrors(l), Error::ValidationErrors(r)) => l == r,
            _ => self.kind() == other.kind(),
        }
    }
}
quick_error! {
    #[derive(Debug, Eq, PartialEq)]
    pub enum ValidationError {
        MultipleZpools(zpools: Vec<PathBuf>) {}
        NameTooLong(dataset: PathBuf) {}
        MissingName(dataset: PathBuf) {}
        MissingSnapshotName(dataset: PathBuf) {}
        MissingPool(daset: PathBuf) {}
        Unknown(dataset: PathBuf) {}
    }
}