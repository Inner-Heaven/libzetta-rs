![RsFreeBSD](libzfs.png)

[![Build Status](https://travis-ci.org/Inner-Heaven/libzfs-rs.svg?branch=master)](https://travis-ci.org/Inner-Heaven/libzfs-rs)
[![codecov](https://codecov.io/gh/Inner-Heaven/libzfs-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/Inner-Heaven/libzfs-rs)
[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2FInner-Heaven%2Flibzfs-rs.svg?type=shield)](https://app.fossa.io/projects/git%2Bgithub.com%2FInner-Heaven%2Flibzfs-rs?ref=badge_shield)

> Use ZFS from Rust. (very experimental)

## Installation
Not yet. Won't be on crate.io until 1.0.

## Usage
Public API for `zpool` interface is almost stable, but until I start work on `zfs` portion I don't want to call it stable.

## How it works
ZFS doesn't have stable API at all. There is `libzfs_core` which supposed to be it, but it really isn't. While `libzfs_core` is somewhat stable `libnvpair` used in it isn't and `libnv` isn't available on Linux. I might embed portable `libnv`. Now the tricky part — `libzfs_core` is just for zfs, there is not `libzpool_core` which means you either have to rely on unstable (in terms of API) `libzpool` or use `zpool(8)`. I decided to use `zpool(8)` because that's a recommended way of doing it.


## Usage
Yeah, not today.

## Current feature status

### zpool

|       | Create | Destroy | Get Properties | Set Properties | Scrub | Export | Import | List Available | Read Status | Add vdev | Replace Disk |
|-------|--------|---------|----------------|----------------|-------|--------|--------|----------------|-------------|----------|--------------|
| open3 |    ✔   |    ✔    |        ✔       |        ✔       |   ❌   |    ✔   |    ✔   |     Limited*    |      ❌      |     ❌    |       ❌      |

### zfs

Literally nothing

## LICENSE

[BSD-2-Clause](LICENSE).


[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2FInner-Heaven%2Flibzfs-rs.svg?type=large)](https://app.fossa.io/projects/git%2Bgithub.com%2FInner-Heaven%2Flibzfs-rs?ref=badge_large)
