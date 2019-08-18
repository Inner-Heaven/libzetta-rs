pub enum DatasetKind {
    Zfs,
    Zvol,
}

pub struct Dataset {
    kind: DatasetKind,
    name: String,
}
