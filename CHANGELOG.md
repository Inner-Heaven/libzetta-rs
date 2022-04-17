# Changelog

All notable changes to this project will be documented in this file.

## [0.3.1] - 2022-04-17

### Bug Fixes

- *(zfs)* Fix destroy_bookmarks method (#170)

### Miscellaneous Tasks
- *(No Category)* Nixify project
- *(No Category)* Add vagrant boxes for testing
- *(No Category)* Add justfile for testing and sunset run_tests.sh
- *(No Category)* Add freebsd12 box


## [0.3.0] - 2022-04-16

### Bug Fixes

- *(zfs)* Unflip bool_to_u64 results
- *(zfs)* ZLE (Zero Length Encoding) compression is not "LZE"
- *(zfs)* Filesystem_limit/snapshot_limit/filesystem_count/snapshot_count can now be 'none' (#163)- *(No Category)* (zpool): "see" line in status is not parsed correctly  (#168)


### Features

- *(crate)* Reexport libnv, do not use unused strum
- *(zfs)* In CreateDatasetRequest, inherit most things if not explicitly set (#155)
- *(zfs)* Add support for running channel programs

### Miscellaneous Tasks

- *(deps)* Update strum requirement from 0.22.0 to 0.23.0 (#160)
- *(deps)* Update strum_macros requirement from 0.22.0 to 0.23.0 (#162)
- *(deps)* Update strum_macros requirement from 0.23.0 to 0.24.0 (#165)
- *(deps)* Update strum requirement from 0.23.0 to 0.24.0 (#164)
- *(deps)* Update derive_builder requirement from 0.10 to 0.11 (#167)- *(No Category)* Remove unused old libzfs_core-sys version
- *(No Category)* Add MVP nix flake support


## [0.2.3] - 2021-10-25

### Bug Fixes

- *(zfs)* Fixed 'Failed to parse value: VariantNotFound' on Linux, zfs-2.0.3 (#146)

### Documentation

- *(changelog)* Update for v0.2.2

### Features

- *(zfs)* Convert nvlist errors to HashMaps to make the Error type Send+Sync (#152) (#153)

### Miscellaneous Tasks

- *(deps)* Update derive_builder requirement from 0.9 to 0.10 (#141)
- *(deps)* Update rand requirement from 0.7 to 0.8 (#143)
- *(deps)* Update strum_macros requirement from 0.18.0 to 0.21.0 (#145)
- *(deps)* Update strum requirement from 0.18.0 to 0.21.0 (#144)
- *(deps)* Update strum requirement from 0.21.0 to 0.22.0 (#147)- *(No Category)* Make it work enough to release


## [0.2.2] - 2020-04-26

### Bug Fixes

- *(zfs)* Fix incremental send in LZC (#128)

### Documentation

- *(readme)* Update to reflect current state of things
- *(readme)* Fix footnote

### Miscellaneous Tasks

- *(changelog)* Update for 0.2.1 (fixed)

### Chose

- *(changelog)* Update for 0.2.1

## [0.2.1] - 2020-03-22

### Bug Fixes
- *(No Category)* Fix tag name...

- *(No Category)* Fix tag name...

- *(No Category)* Fix license shield in readme



### Features

- *(zfs)* Fix dataset name parser for ZFS (#127)

### Miscellaneous Tasks

- *(deps)* Update strum requirement from 0.17.1 to 0.18.0 (#125)
- *(deps)* Update strum_macros requirement from 0.17.1 to 0.18.0 (#124)

## [0.2.0] - 2020-03-01

### Bug Fixes

- *(zpool)* Integer overflow in zpool parser

### Features

- *(zfs)* Check existence of dataset.
- *(zfs)* Basic zfs create and destroy operations (#91)
- *(zfs)* Listing filesystems and volumes (#94)
- *(zfs)* Add PathExt trait to make it easier to work with dataset names (#100)
- *(zfs)* Pass errors from lzc snapshot call to the consumer. (#102)
- *(zfs)* Ability to read filesystem dataset properties (#111)
- *(zfs)* Read properties of a snapshot (#112)
- *(zfs)* Read properties of a volume (#113)
- *(zfs)* Read properties of a bookmark (#114)
- *(zfs)* Ability to work with bookmarks
- *(zfs)* Ability to send snapshot (#119)
- *(zfs)* Remove known unknowns from properties (#121)- *(No Category)* Inception of ZFS module (#83)
- *(No Category)* Fuzzy testing target (#90)
- *(No Category)* Remove unicode feature from regex crate (#93)
- *(No Category)* Add a single point of logging configuration (#123)


### Miscellaneous Tasks

- *(deps)* Update slog-stdlog requirement from 3 to 4 (#79)
- *(deps)* Update getset requirement from 0.0.7 to 0.0.8 (#92)
- *(deps)* Update strum requirement from 0.15.0 to 0.16.0 (#98)
- *(deps)* Update strum_macros requirement from 0.15.0 to 0.16.0 (#97)
- *(deps)* Update derive_builder requirement from 0.7 to 0.8 (#105)
- *(deps)* Update getset requirement from 0.0.8 to 0.0.9 (#106)
- *(deps)* Update derive_builder requirement from 0.8 to 0.9 (#107)
- *(deps)* Update strum requirement from 0.16.0 to 0.17.1 (#108)
- *(deps)* Update strum_macros requirement from 0.16.0 to 0.17.1 (#109)
- *(deps)* Update getset requirement from 0.0.9 to 0.1.0 (#116)- *(No Category)* Grammar and other documentation fixes (#75)
- *(No Category)* Update logo for the rename
- *(No Category)* Add Changelog
- *(No Category)* Testing out different release tools
- *(No Category)* Update issue templates
- *(No Category)* Update readme and Cargo.toml


### Styling
- *(No Category)* Run cargo fmt (#117)


## [0.1.2] - 2019-08-12

### Miscellaneous Tasks
- *(No Category)* Rename project to libzetta (#74)


## [0.1.1] - 2019-08-12

### Bug Fixes
- *(No Category)* Fix(libnv) forgot to make them public

- *(No Category)* Fix(libnv) proper library name + version bump

- *(No Category)* Fix(libnv) fix some method signatures

- *(No Category)* Fix(libnv) fix some method signatures again

- *(No Category)* Fix(nv) Remove most of the .expect() calls

From now on failure to convert str to cstring will no longer panic and instead return a proper error

- *(No Category)* Fix codecov badge


### Features

- *(zpool)* Add Zpool::add (#53)
- *(zpool)* Remove device from zpool (#60)
- *(zpool)* Fix parser for logs and caches. Add add_zil and add_cache (#63)
- *(zpool)* Add replace_disk. Closes #25 (#67)
- *(zpool)* Add regex for another type of vdev reuse. Closes #49 (#69)- *(No Category)* Feat(libnv) add unsafe FFI bindings

- *(No Category)* Feat(libnv) add unsafe FFI bindings

- *(No Category)* Feat(libnv) Proper license name

- *(No Category)* Feat(libzfs) initial ffi bindings to libzfs_core

- *(No Category)* Feat(libzfs) add ffi deps

- *(No Category)* Feat(nv) Safe wrapper around libnv

- *(No Category)* Feat(project) add rust fmt config.

- *(No Category)* Feature(zpool) add some validators to Topology [wip] (#3)

* feature(zpool) add some validators to Topology

- *(No Category)* Feat(zpool) into_args

- *(No Category)* Feat(zpool) Better error handling.

Improved error handling and add a whole lot of tests. Also made tests run on Travis-ci without issues.
- *(No Category)* Feat(zpool) read properties of zpool (#6)

* feat(zpool) read properties of zpool
- *(No Category)* Feat(zpool) update zpool props (#10)


- *(No Category)* Feat(zpool) import/export and reduced disk usage

* feat(test) Reduce disk usage during test.



### Miscellaneous Tasks

- *(deps)* Update derive-getters requirement from 0.0.7 to 0.0.8 (#52)
- *(deps)* Update rand requirement from 0.6 to 0.7 (#57)
- *(zpool)* Big update to docs and cleanup all public API. (#71)- *(No Category)* Chore(libnv) cargo metadata

- *(No Category)* Chore(libnv) bump version

- *(No Category)* Chore(travis) Initial config. Sure, it will work.

- *(No Category)* Chore(libnv) change library name to link against

- *(No Category)* Chore(travis) well, screw you too. travis-ci/travis-ci#1818

- *(No Category)* Chore(travis) try to add it back

For now zfs module will only work on freebsd

- *(No Category)* Chore(travis) forgot this

- *(No Category)* Chore(travis) I swear, I've done this already.

- *(No Category)* Chore(travis) make project ready again

- *(No Category)* Chore(travis) make project ready again

- *(No Category)* Chore(travis) add codecov

- *(No Category)* Chore(zpool) one more test case missing

- *(No Category)* Chore(zpool) cleanup some code, make clippy not angry at me (#9)


- *(No Category)* Chore(zpool) cleanup

- *(No Category)* Chore(deps) Update rand (#11)

* chore(deps) Bump some dependencies

- *(No Category)* Try using cargo-suity for test reports (#40)


### Refactor
- *(No Category)* Make Vdev and Zpool structure more understandable (#39)
- *(No Category)* Switch to Pairs#as_span (#56)


### Styling
- *(No Category)* Style(nv) Run rustfmt on a project

- *(No Category)* Style(project) run cargo fmt

- *(No Category)* Style(clippy) make clippy happy

- *(No Category)* Style(fmt) Add a few test cases and run cargo fmt

- *(No Category)* New fmt config (#54)


### Testing
- *(No Category)* Tests require sudo



<!-- generated by git-cliff -->
