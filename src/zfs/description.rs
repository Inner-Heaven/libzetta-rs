use strum_macros::{AsRefStr, Display, EnumString};
use std::default::Default;

#[derive(AsRefStr, EnumString, Display, Eq, PartialEq, Debug, Clone)]
pub enum DatasetKind {
    #[strum(serialize = "filesystem")]
    Filesystem,
    #[strum(serialize = "volume")]
    Volume,
    #[strum(serialize = "snapshot")]
    Snapshot,
}

impl Default for DatasetKind {
    fn default() -> Self {
        DatasetKind::Filesystem
    }
}

impl DatasetKind {
    pub fn as_nvpair_value(&self) -> &str {
        match &self {
            DatasetKind::Filesystem => "zfs",
            DatasetKind::Volume => "zvol",
            _ => panic!("Unsupported dataset kind")
        }
    }
}

pub struct Dataset {
    kind: DatasetKind,
    name: String,
}
