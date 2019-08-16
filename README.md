<p align="center">
  <img src="libzetta.png">
</p>

[![Build Status](https://dev.azure.com/andoriyu/libpandemonium/_apis/build/status/libzetta-rs?branchName=master)](https://dev.azure.com/andoriyu/libpandemonium/_build/latest?definitionId=4&branchName=master)
[![codecov](https://codecov.io/gh/Inner-Heaven/libzetta-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/Inner-Heaven/libzetta-rs)
[![Crates.io](https://img.shields.io/crates/v/libzetta.svg)](https://crates.io/crates/libzetta)
[![Cirrus CI - Base Branch Build Status](https://img.shields.io/cirrus/github/Inner-Heaven/libzetta-rs?label=cirrus-ci)](https://cirrus-ci.com/github/Inner-Heaven/libzetta-rs)
[![docs.rs](https://docs.rs/libzetta/badge.svg)](https://docs.rs/libzetta)
> libzetta-rs is a stable interface for programmatic administration of ZFS

## Installation
Not yet. It won't destroy your pool, but API might change any moment. Wait until 1.0.0. I have a pretty decent roadmap to 1.0.0.

## Usage
Public API for `zpool` interface is almost at the point where I'm going to stabilize it, but until I start work on `zfs` portion I don't want to call it stable.

### FreeBSD
This library mostly focused on FreeBSD support. This should work on any FreeBSD version since 9.2. However, I have no intention on supporting anything other than current releases. Yes, I know FreeBSD is switching to ZOL branch.

### Linux
Right now it definitely works with `0.7.2` maybe entire `0.7.x` branch. Linux support is minimum effort - if I upgrade zfs to the version and suddenly all tests are failing - I'm going to rollback and lock previous version.

## How it works
ZFS doesn't have stable API at all. There is `libzfs_core` which supposed to be it, but it really isn't. While `libzfs_core` is somewhat stable `libnvpair` used in it isn't and `libnv` isn't available on Linux. I might embed portable `libnv`. Now the tricky part — `libzfs_core` is just for zfs, there is not `libzpool_core` which means you either have to rely on unstable (in terms of API) `libzpool` or use `zpool(8)`. I decided to use `zpool(8)` because that's a recommended way of doing it.

## Running tests

Note that integration tests do a lot of zpool and zfs operations on live system. I recommend spin up a VM and use `run_tests.sh` to run integration tests in side that VM. Tests also take a lot of disk space because each vdev is at least 64mb file.

## Current feature status

### zpool

|       | Create | Destroy | Get Properties | Set Properties | Scrub | Export | Import | List Available | Read Status | Add vdev | Replace Disk |
|-------|--------|---------|----------------|----------------|-------|--------|--------|----------------|-------------|----------|--------------|
| open3 |    ✔   |    ✔    |        ✔       |        ✔       |   ✔   |    ✔   |    ✔   |     ✔    |      ✔ ¹     |     ✔    |       ❌      |

1. Reads the status, but api isn't stable and does poor job at reporting scrubbing status.


### zfs

Literally nothing works.

## LICENSE

[BSD-2-Clause](LICENSE).
