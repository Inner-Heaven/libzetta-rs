use crate::{slog::Logger,
            zfs::{lzc::ZfsLzc, open3::ZfsOpen3, BookmarkRequest, CreateDatasetRequest,
                  DatasetKind, DestroyTiming, Properties, Result, ZfsEngine}};
use std::{collections::HashMap, path::PathBuf};

/// Handy wrapper that delegates your call to correct implementation.
pub struct DelegatingZfsEngine {
    lzc:   ZfsLzc,
    open3: ZfsOpen3,
}

impl DelegatingZfsEngine {
    pub fn new(root_logger: Option<Logger>) -> Result<Self> {
        let lzc = ZfsLzc::new(root_logger.clone())?;
        let open3 = ZfsOpen3::new(root_logger);
        Ok(DelegatingZfsEngine { lzc, open3 })
    }
}

impl ZfsEngine for DelegatingZfsEngine {
    fn exists<N: Into<PathBuf>>(&self, name: N) -> Result<bool> { self.lzc.exists(name) }

    fn create(&self, request: CreateDatasetRequest) -> Result<()> { self.lzc.create(request) }

    fn snapshot(
        &self,
        snapshots: &[PathBuf],
        user_properties: Option<HashMap<String, String>>,
    ) -> Result<()> {
        self.lzc.snapshot(snapshots, user_properties)
    }

    fn bookmark(&self, bookmarks: &[BookmarkRequest]) -> Result<()> { self.lzc.bookmark(bookmarks) }

    fn destroy<N: Into<PathBuf>>(&self, name: N) -> Result<()> { self.open3.destroy(name) }

    fn destroy_snapshots(&self, snapshots: &[PathBuf], timing: DestroyTiming) -> Result<()> {
        self.lzc.destroy_snapshots(snapshots, timing)
    }

    fn destroy_bookmarks(&self, bookmarks: &[PathBuf]) -> Result<()> {
        self.lzc.destroy_bookmarks(bookmarks)
    }

    fn list<N: Into<PathBuf>>(&self, pool: N) -> Result<Vec<(DatasetKind, PathBuf)>> {
        self.open3.list(pool)
    }

    fn list_filesystems<N: Into<PathBuf>>(&self, pool: N) -> Result<Vec<PathBuf>> {
        self.open3.list_filesystems(pool)
    }

    fn list_snapshots<N: Into<PathBuf>>(&self, pool: N) -> Result<Vec<PathBuf>> {
        self.open3.list_snapshots(pool)
    }

    fn list_bookmarks<N: Into<PathBuf>>(&self, pool: N) -> Result<Vec<PathBuf>> {
        self.open3.list_bookmarks(pool)
    }

    fn list_volumes<N: Into<PathBuf>>(&self, pool: N) -> Result<Vec<PathBuf>> {
        self.open3.list_volumes(pool)
    }

    fn read_properties<N: Into<PathBuf>>(&self, path: N) -> Result<Properties> {
        self.open3.read_properties(path)
    }
}
