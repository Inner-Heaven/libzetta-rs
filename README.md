![RsFreeBSD](libzfs.png)

[![Build Status](https://dev.azure.com/andoriyu/libpandemonium/_apis/build/status/Inner-Heaven.libzfs-rs?branchName=master)](https://dev.azure.com/andoriyu/libpandemonium/_build/latest?definitionId=1&branchName=master)
[![codecov](https://codecov.io/gh/Inner-Heaven/libzfs-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/Inner-Heaven/libzfs-rs)
[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2FInner-Heaven%2Flibzfs-rs.svg?type=shield)](https://app.fossa.io/projects/git%2Bgithub.com%2FInner-Heaven%2Flibzfs-rs?ref=badge_shield)

> libzfs-rs is a stable interface for programmatic administration of ZFS

## Installation
Not yet. Won't be on crate.io until 0.2.0. Even then it will be very alpha. Wait for 1.0.1 which when it will have stable API, integrated into another libpandemonium project, properly documented, etc.

## Usage
Public API for `zpool` interface is almost at the point where I'm going to stabilize it, but until I start work on `zfs` portion I don't want to call it stable. 

### FreeBSD
This library mostly focused on FreeBSD support. This should work on any FreeBSD version since 9.2. However, I have no intention on supporting anything other than current releases. Yes, I know FreeBSD is switching to ZOL branch.

### Linux
Right now it definitely works with `0.7.2` maybe entire `0.7.x` branch. Only reason there is Linux support is because there is no free public CI that has FreeBSD executors. Linux support is minimum effort - if I upgrade zfs to the version and suddenly all tests are failing - I'm going to rollback and lock previous version.

## How it works
ZFS doesn't have stable API at all. There is `libzfs_core` which supposed to be it, but it really isn't. While `libzfs_core` is somewhat stable `libnvpair` used in it isn't and `libnv` isn't available on Linux. I might embed portable `libnv`. Now the tricky part — `libzfs_core` is just for zfs, there is not `libzpool_core` which means you either have to rely on unstable (in terms of API) `libzpool` or use `zpool(8)`. I decided to use `zpool(8)` because that's a recommended way of doing it.

## Running tests

Note that integration tests do a lot of zpool and zfs operations on live system. I recommend spin up a VM and use `run_tests.sh` to run integration tests in side that VM. Tests also take a lot of disk space because each vdev is at least 64mb file. 

## Current feature status

### zpool

|       | Create | Destroy | Get Properties | Set Properties | Scrub | Export | Import | List Available | Read Status | Add vdev | Replace Disk |
|-------|--------|---------|----------------|----------------|-------|--------|--------|----------------|-------------|----------|--------------|
| open3 |    ✔   |    ✔    |        ✔       |        ✔       |   ✔   |    ✔   |    ✔   |     ✔    |      ✔ ¹     |     ❌    |       ❌      |

1. Reads the status, but api isn't stable and does poor job at reporting scrubbing status.


### zfs

Literally nothing works.

## LICENSE

[BSD-2-Clause](LICENSE).


[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2FInner-Heaven%2Flibzfs-rs.svg?type=large)](https://app.fossa.io/projects/git%2Bgithub.com%2FInner-Heaven%2Flibzfs-rs?ref=badge_large)
