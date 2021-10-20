use crate::zfs::{lzc::ZfsLzc, open3::ZfsOpen3, BookmarkRequest, CreateDatasetRequest, DatasetKind,
                 DestroyTiming, Properties, Result, SendFlags, ZfsEngine};
use std::{collections::HashMap, os::unix::io::AsRawFd, path::PathBuf};

/// Handy wrapper that delegates your call to correct implementation.
pub struct DelegatingZfsEngine {
    lzc:   ZfsLzc,
    open3: ZfsOpen3,
}

impl DelegatingZfsEngine {
    pub fn new() -> Result<Self> {
        let lzc = ZfsLzc::new()?;
        let open3 = ZfsOpen3::new();
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

    fn send_full<N: Into<PathBuf>, FD: AsRawFd>(
        &self,
        path: N,
        fd: FD,
        flags: SendFlags,
    ) -> Result<()> {
        self.lzc.send_full(path, fd, flags)
    }

    fn send_incremental<N: Into<PathBuf>, F: Into<PathBuf>, FD: AsRawFd>(
        &self,
        path: N,
        from: F,
        fd: FD,
        flags: SendFlags,
    ) -> Result<()> {
        self.lzc.send_incremental(path, from, fd, flags)
    }

    fn run_channel_program<N: Into<PathBuf>>(
        &self,
        pool: N,
        program: &str,
        instr_limit: u64,
        mem_limit: u64,
        sync: bool,
        args: libnv::nvpair::NvList,
    ) -> Result<libnv::nvpair::NvList> {
        self.lzc.run_channel_program(pool, program, instr_limit, mem_limit, sync, args)
    }
}
