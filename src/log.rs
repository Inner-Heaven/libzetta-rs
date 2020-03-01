use bitflags::_core::ops::Deref;
use once_cell::sync::OnceCell;
use slog::{Drain, Logger as SlogLogger};
use slog_stdlog::StdLog;
use std::borrow::Borrow;

static GLOBAL_LOGGER: OnceCell<Logger> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct Logger {
    inner: SlogLogger,
}

impl Deref for Logger {
    type Target = SlogLogger;

    fn deref(&self) -> &Self::Target { self.inner.borrow() }
}

impl Logger {
    fn new(logger: SlogLogger) -> Self { Logger { inner: logger } }

    /// Get global logger.
    pub fn global() -> &'static Logger {
        GLOBAL_LOGGER.get_or_init(|| Logger::new(SlogLogger::root(StdLog.fuse(), o!())))
    }

    /// Can only called once. Returns Ok(()) if the cell was empty and Err(value) if it was full.
    pub fn setup(root_logger: SlogLogger) -> Result<(), Logger> {
        GLOBAL_LOGGER.set(Logger::new(root_logger))
    }
}
