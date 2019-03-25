#![recursion_limit = "256"]

#[cfg(test)]
extern crate cavity;
#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate derive_getters;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate pest;
extern crate pest_derive;
#[macro_use]
extern crate quick_error;
extern crate regex;
#[macro_use]
pub extern crate slog;
extern crate slog_stdlog;
#[cfg(test)]
extern crate tempdir;

// library modules
pub mod parsers;
pub mod zpool;
