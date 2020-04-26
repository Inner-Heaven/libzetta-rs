<p align="center">
  <img src="libzetta.png">
</p>

[![Build Status](https://dev.azure.com/andoriyu/libpandemonium/_apis/build/status/libzetta-rs?branchName=master)](https://dev.azure.com/andoriyu/libpandemonium/_build/latest?definitionId=4&branchName=master)
[![codecov](https://codecov.io/gh/Inner-Heaven/libzetta-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/Inner-Heaven/libzetta-rs)
[![Crates.io](https://img.shields.io/crates/v/libzetta.svg)](https://crates.io/crates/libzetta)
[![Cirrus CI - Base Branch Build Status](https://img.shields.io/cirrus/github/Inner-Heaven/libzetta-rs?label=cirrus-ci)](https://cirrus-ci.com/github/Inner-Heaven/libzetta-rs)
[![docs.rs](https://docs.rs/libzetta/badge.svg)](https://docs.rs/libzetta)
[![license](https://img.shields.io/github/license/Inner-Heaven/libzetta-rs)](https://github.com/Inner-Heaven/libzetta-rs/blob/master/LICENSE)

> libzetta-rs is a stable interface for programmatic administration of ZFS

## Installation

Not yet. It won't break your pool or kill your brother, but API might change. Wait until 1.0.0. I have a pretty decent roadmap to 1.0.0.

## Usage

Public API for `zpool` stable. Public API for `zfs` might change after I actually get to use it in other projects. Consult the documention on usage.

### FreeBSD

This library focused on FreeBSD support. This should work on any FreeBSD version since 9.2. No intention on supporting legacy versions. Supported versions:
 - 12.1

### Linux

Right now it verified works with `0.7.13` and entire `0.7.x` branch.

## How it works

ZFS doesn't have stable API at all.`libzfs_core`(`lzc`) fills some gaps, but not entirely. While `lzc` provides stable APi to some features of zfs, there is no such thing for zpool. This library resorts to `zfs(8)` and `zpool(8)` where `lzc` falls shorts.

## Running tests

Note that integration tests do a lot of zpool and zfs operations on live system. I recommend spin up a VM and use `run_tests.sh` to run integration tests inside. Tests also take a lot of disk space because each vdev is at least 64mb file.

## Current feature status

### zpool

|       | Create | Destroy | Get Properties | Set Properties | Scrub | Export | Import | List Available | Read Status | Add vdev | Replace Disk |
|-------|--------|---------|----------------|----------------|-------|--------|--------|----------------|-------------|----------|--------------|
| open3 | ✔      | ✔       | ✔              | ✔              | ✔     | ✔      | ✔      | ✔              | ✔¹         | ✔        | ✔            |

1. Reads the status, but api isn't stable and does poor job at reporting scrubbing status.


### zfs

#### Filesystem and ZVOL

|         | Create    | Destroy     | List     | Get Properties    | Update Properties     |
| ------- | --------- | ----------- | -------- | ----------------- | --------------------- |
| open3   | ❌        | ❌          | ✔        | ✔                 | ❌                    |
| lzc     | ✔¹        | ✔           | ❌       | ❌                | ❌                    |

1. Might not have all properties available.

#### Snapshot and bookmark

|       	|  Create 	|   Destroy 	|   List 	|  Get Properties 	|   Send 	| Recv 	|
|-------	|---------	|-----------	|--------	|-----------------	|--------	|------	|
| open3 	| ❌       	| ❌         	| ✔      	| ✔               	| ❌      	| ❌    	|
|  lzc  	| ✔¹     	| ✔         	| ❌      	| ❌               	| ✔      	| ❌    	|

1. Might not have all properties available.

## Alternatives

### https://github.com/whamcloud/rust-libzfs

Unlike them LibZetta doesn't link against private libraries of ZFS. `libzetta` also has more documention.

### https://github.com/jmesmon/rust-libzfs

LibZetta has zpool APIs. LibZetta shares `-sys` crates with this library. LibZetta also will delegate certain features of `zfs(8)` to open3 implementation.

## LICENSE

[BSD-2-Clause](LICENSE).
