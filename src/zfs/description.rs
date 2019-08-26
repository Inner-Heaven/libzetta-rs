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

pub struct Dataset {
    kind: DatasetKind,
    name: String,
}
