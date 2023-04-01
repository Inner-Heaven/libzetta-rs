#![recursion_limit = "256"]
#![deny(
    nonstandard_style,
    future_incompatible,
    clippy::all,
    clippy::restriction,
    clippy::nursery
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::multiple_inherent_impl,
    clippy::implicit_return,
    clippy::missing_inline_in_public_items,
    clippy::missing_docs_in_private_items
)]

//! Rust bindings to libzfs_core and wrapper around `zpool(8)`.
//!
//! This library intends to provide a safe, low-level interface to ZFS operator tools. As such, not
//! much will be sugar coated here.
//!
//! # Overview
//! ## zpool
//! A feature complete wrapper around `zpool(8)` with a somewhat stable API. I can't
//! guarantee that the API won't change at any moment, but I don't see a reason for it change at the
//! moment.
//!
//! Refer to the [zpool module documentation](zpool/index.html) for more information.
//!
//! ## zfs
//! Most of functionality of `libzfs_core` is covered with some gaps filled in by `open3`.
//!
//! Refer to the [zfs module documentation](zfs/index.html) for more information.
//!
//! # Usage
//!
//! This section is currently under contstruction. Meanwhile, look at integration tests for
//! inspiration.
//!
//! # Project Structure
//! ### parsers
//! Module for PEG parsers backed by [Pest](https://pest.rs/).
//!
//! ### zpool
//! This module contains everything you need to work with zpools.

#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate getset;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quick_error;

#[macro_use]
pub extern crate slog;
pub use pest;

pub extern crate libnv;

// library modules
pub mod parsers;
pub mod zfs;
pub mod zpool;

pub mod utils;

#[cfg(fuzzing)]
pub mod fuzzy;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod log;
pub use log::GlobalLogger;

pub mod fuckery {
    extern "C" {
        pub(crate) fn fuckery_make_nvlist() -> *mut zfs_core_sys::nvlist_t;
    }
}
