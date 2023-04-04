use std::ops::Deref;
use once_cell::sync::OnceCell;
use slog::{Drain, Logger as SlogLogger};
use slog_stdlog::StdLog;
use std::borrow::Borrow;

static GLOBAL_LOGGER: OnceCell<GlobalLogger> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct GlobalLogger {
    inner: SlogLogger,
}

impl Deref for GlobalLogger {
    type Target = SlogLogger;

    fn deref(&self) -> &Self::Target {
        self.inner.borrow()
    }
}

impl GlobalLogger {
    fn new(logger: SlogLogger) -> Self {
        GlobalLogger { inner: logger }
    }

    /// Get global logger. If you didn't call `Logger::setup` prior calling this then default logger
    /// created with `StdLog` as drain.
    pub fn get() -> &'static GlobalLogger {
        GLOBAL_LOGGER.get_or_init(|| {
            let root_logger = SlogLogger::root(StdLog.fuse(), o!());
            GlobalLogger::new(logger_from_root_logger(&root_logger))
        })
    }

    /// Set global logger. Optional.
    /// Can only called once. Returns Ok(()) if the cell was empty and Err(value) if it was full.
    pub fn setup(root_logger: &SlogLogger) -> Result<(), GlobalLogger> {
        GLOBAL_LOGGER.set(GlobalLogger::new(logger_from_root_logger(root_logger)))
    }
}

fn logger_from_root_logger(root_logger: &SlogLogger) -> SlogLogger {
    root_logger.new(o!("zetta_version" => crate::VERSION))
}
