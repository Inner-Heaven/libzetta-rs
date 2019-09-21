use crate::zfs::ValidationResult;
use std::path::Path;

pub trait PathExt {
    fn get_pool(&self) -> Option<String>;
    fn get_snapshot(&self) -> Option<String>;
    fn get_bookmark(&self) -> Option<String>;

    fn is_snapshot(&self) -> bool { self.get_snapshot().is_some() }
    fn is_bookmark(&self) -> bool { self.get_bookmark().is_some() }
    fn is_volume_or_dataset(&self) -> bool { !self.is_bookmark() && !self.is_snapshot() }

    fn is_valid(&self) -> bool {
        if let Ok(()) = self.validate() {
            true
        } else {
            false
        }
    }

    fn validate(&self) -> ValidationResult;
}

impl PathExt for Path {
    fn get_pool(&self) -> Option<String> {
        if self.has_root() || self.components().count() < 2 {
            return None;
        }
        if let Some(root) = self.iter().next() {
            Some(root.to_string_lossy().to_string())
        } else {
            None
        }
    }

    fn get_snapshot(&self) -> Option<String> {
        if let Some(last) = self.file_name() {
            let as_str = last.to_string_lossy();
            if as_str.contains('@') {
                return as_str.rsplit('@').next().map(String::from);
            }
        }
        None
    }

    fn get_bookmark(&self) -> Option<String> {
        if let Some(last) = self.file_name() {
            let as_str = last.to_string_lossy();
            if as_str.contains('#') {
                return as_str.rsplit('#').next().map(String::from);
            }
        }
        None
    }

    fn validate(&self) -> ValidationResult { crate::zfs::validators::validate_name(self) }
}

impl<P: AsRef<Path>> PathExt for P {
    fn get_pool(&self) -> Option<String> { self.as_ref().get_pool() }

    fn get_snapshot(&self) -> Option<String> { self.as_ref().get_snapshot() }

    fn get_bookmark(&self) -> Option<String> { self.as_ref().get_snapshot() }

    fn validate(&self) -> ValidationResult { self.as_ref().validate() }
}

#[cfg(test)]
mod test {
    use super::PathExt;
    use std::path::PathBuf;

    #[test]
    fn valid_dataset_no_bookmarks_or_snapshots() {
        let path = PathBuf::from("tank/usr/home");

        assert_eq!(Some(String::from("tank")), path.get_pool());
        assert!(!path.is_snapshot());
        assert!(!path.is_bookmark());
        assert_eq!(None, path.get_snapshot());
        assert_eq!(None, path.get_bookmark());
        assert!(path.is_volume_or_dataset());
        assert!(path.is_valid());
    }

    #[test]
    fn not_valid_just_dataset() {
        let path = PathBuf::from("/usr/home");
        assert_eq!(None, path.get_pool());
        assert!(!path.is_snapshot());
        assert!(!path.is_bookmark());
        assert_eq!(None, path.get_snapshot());
        assert_eq!(None, path.get_bookmark());
        assert!(path.is_volume_or_dataset());
        assert!(!path.is_valid());
    }

    #[test]
    fn valid_snapshot() {
        let path = PathBuf::from("tank/usr/home@snap");

        assert_eq!(Some(String::from("tank")), path.get_pool());
        assert!(path.is_snapshot());
        assert!(!path.is_bookmark());
        assert_eq!(Some(String::from("snap")), path.get_snapshot());
        assert_eq!(None, path.get_bookmark());
        assert!(!path.is_volume_or_dataset());
        assert!(path.is_valid());
    }
    #[test]
    fn valid_bookmark() {
        let path = PathBuf::from("tank/usr/home#bookmark");

        assert_eq!(Some(String::from("tank")), path.get_pool());
        assert!(!path.is_snapshot());
        assert!(path.is_bookmark());
        assert_eq!(None, path.get_snapshot());
        assert_eq!(Some(String::from("bookmark")), path.get_bookmark());
        assert!(!path.is_volume_or_dataset());
        assert!(path.is_valid());
    }

    #[test]
    fn at_in_wrong_place() {
        let path = PathBuf::from("tank/usr@wat/home");
        assert!(!path.is_snapshot());
    }

    #[test]
    fn pound_in_wrong_place() {
        let path = PathBuf::from("tank/usr#wat/home");
        assert!(!path.is_bookmark());
    }
}
