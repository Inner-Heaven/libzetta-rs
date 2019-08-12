#![recursion_limit = "256"]
#![deny(nonstandard_style, future_incompatible, clippy::all, clippy::restriction, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

//! Rust bindings to libzfs_core and wrapper around `zpool(8).
//!
//! This library intends to provide a safe interface to ZFS operator tools. This library also meant
//! to be a low level library not much will be sugar coated here.
//!
//! # Overview
//! ## zpool
//! Library has a feature complete wrapper around `zpool(8)` with somewhat stable api. I can't guarantee
//! that API won't change any moment, but I don't see a reason for it be changes at the moment.
//!
//! Refer to [zpool module documentation](zpool/index.html) for more information.
//!
//! ## zfs
//! Work on bindings to `libzfs_core` is just starting, so support for it is non-existent at the moment.
//!
//! # Usage
//! Right now there is no "library usage" instruction, but zpool module can be used directly.
//! In the future some sugar to setup logging will be added to library level.
//!
//! # Project structure
//! ### parsers
//! Module for PEG parsers backed by [Pest](https://pest.rs/).
//!
//! ### zpool
//! Module contain everything you need to work with zpool besides the parsers themselves. However,
//! actual conversion of [Pair](../pest/iterators/struct.Pair.html) happens here.
//!
//! ### zfs
//! Doesn't exist yet. I don't what will go there.


#[macro_use] extern crate derive_builder;
#[macro_use] extern crate getset;

#[macro_use]
extern crate lazy_static;

use pest;

#[macro_use]
extern crate quick_error;

#[macro_use]
pub extern crate slog;



// library modules
pub mod parsers;
pub mod zpool;
