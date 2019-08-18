use cstr_argument::CStrArgument;

pub mod description;
pub use description::{Dataset, DatasetKind};

pub mod lzc;
pub use lzc::ZfsLzc;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        ZFSInitializationFailed(err: std::io::Error) {
            cause(err)
        }
        Io(err: std::io::Error) {
            cause(err)
        }
    }
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub trait ZfsEngine {
    /// Check if a dataset (a filesystem, or a volume, or a snapshot with the given name exists.
    ///
    /// NOTE: Can't be used to check for existence of bookmarks.
    ///  * `name` - The dataset name to check.
    fn exists<D: CStrArgument>(&self, name: D) -> Result<bool>;

    //fn list(pool: Option<String>) -> Result<Vec<Dataset>>;
}
