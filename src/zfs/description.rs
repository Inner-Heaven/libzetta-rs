use std::default::Default;
use strum_macros::{AsRefStr, Display, EnumString};

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
    fn default() -> Self { DatasetKind::Filesystem }
}

impl DatasetKind {
    pub fn as_c_uint(&self) -> zfs_core_sys::lzc_dataset_type::Type {
        match self {
            DatasetKind::Filesystem => zfs_core_sys::lzc_dataset_type::LZC_DATSET_TYPE_ZFS,
            DatasetKind::Volume => zfs_core_sys::lzc_dataset_type::LZC_DATSET_TYPE_ZVOL,
            _ => panic!("Not supported"),
        }
    }

    pub fn as_nvpair_value(&self) -> &str {
        match &self {
            DatasetKind::Filesystem => "zfs",
            DatasetKind::Volume => "zvol",
            _ => panic!("Unsupported dataset kind"),
        }
    }
}

pub struct Dataset {
    kind: DatasetKind,
    name: String,
}
