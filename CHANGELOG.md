# Changelog

All notable changes to this project will be documented in this file.

## [0.4.1] - 2023-04-01

### Bug Fixes

- Fix solaris build by using forked zfs-core-sys ([#185](https://github.com/ZeroAssumptions/aide-de-camp/issues/185))

## [0.4.0] - 2023-04-01

### Bug Fixes

- **zfs:** Allow snapshotting of entire pool ([#173](https://github.com/ZeroAssumptions/aide-de-camp/issues/173))
- **zpool:** Handling of in-use spares

### Documentation

- **README:** Bring it more closer to reality. ([#171](https://github.com/ZeroAssumptions/aide-de-camp/issues/171))

## [0.3.1] - 2022-04-17

### Bug Fixes

- **zfs:** Fix destroy_bookmarks method ([#170](https://github.com/ZeroAssumptions/aide-de-camp/issues/170))

### Ci

- Disable bookmarking test for now

## [0.3.0] - 2022-04-16

### Bug Fixes

- **zfs:** Unflip bool_to_u64 results
- **zfs:** ZLE (Zero Length Encoding) compression is not "LZE"
- **zfs:** Filesystem_limit/snapshot_limit/filesystem_count/snapshot_count can now be 'none' ([#163](https://github.com/ZeroAssumptions/aide-de-camp/issues/163))
- (zpool): "see" line in status is not parsed correctly  ([#168](https://github.com/ZeroAssumptions/aide-de-camp/issues/168))

### Features

- **zfs:** In CreateDatasetRequest, inherit most things if not explicitly set ([#155](https://github.com/ZeroAssumptions/aide-de-camp/issues/155))
- **zfs:** Add support for running channel programs
- **crate:** Reexport libnv, do not use unused strum

### Ci

- **azure:** Bump builders to 20.04 ([#158](https://github.com/ZeroAssumptions/aide-de-camp/issues/158))

## [0.2.3] - 2021-10-25

### Bug Fixes

- **zfs:** Fixed 'Failed to parse value: VariantNotFound' on Linux, zfs-2.0.3 ([#146](https://github.com/ZeroAssumptions/aide-de-camp/issues/146))

### Documentation

- **changelog:** Update for v0.2.2

### Features

- **zfs:** Convert nvlist errors to HashMaps to make the Error type Send+Sync ([#152](https://github.com/ZeroAssumptions/aide-de-camp/issues/152)) ([#153](https://github.com/ZeroAssumptions/aide-de-camp/issues/153))

## [0.2.2] - 2020-04-26

### Bug Fixes

- **zfs:** Fix incremental send in LZC ([#128](https://github.com/ZeroAssumptions/aide-de-camp/issues/128))

### Documentation

- **readme:** Update to reflect current state of things
- **readme:** Fix footnote

### Chose

- **changelog:** Update for 0.2.1

## [0.2.1] - 2020-03-22

### Features

- **zfs:** Fix dataset name parser for ZFS ([#127](https://github.com/ZeroAssumptions/aide-de-camp/issues/127))

## [0.2.0] - 2020-03-01

### Bug Fixes

- **zpool:** Integer overflow in zpool parser

### Features

- Inception of ZFS module ([#83](https://github.com/ZeroAssumptions/aide-de-camp/issues/83))
- **zfs:** Check existence of dataset.
- Fuzzy testing target ([#90](https://github.com/ZeroAssumptions/aide-de-camp/issues/90))
- **zfs:** Basic zfs create and destroy operations ([#91](https://github.com/ZeroAssumptions/aide-de-camp/issues/91))
- Remove unicode feature from regex crate ([#93](https://github.com/ZeroAssumptions/aide-de-camp/issues/93))
- **zfs:** Listing filesystems and volumes ([#94](https://github.com/ZeroAssumptions/aide-de-camp/issues/94))
- **zfs:** Add PathExt trait to make it easier to work with dataset names ([#100](https://github.com/ZeroAssumptions/aide-de-camp/issues/100))
- **zfs:** Pass errors from lzc snapshot call to the consumer. ([#102](https://github.com/ZeroAssumptions/aide-de-camp/issues/102))
- **zfs:** Ability to read filesystem dataset properties ([#111](https://github.com/ZeroAssumptions/aide-de-camp/issues/111))
- **zfs:** Read properties of a snapshot ([#112](https://github.com/ZeroAssumptions/aide-de-camp/issues/112))
- **zfs:** Read properties of a volume ([#113](https://github.com/ZeroAssumptions/aide-de-camp/issues/113))
- **zfs:** Read properties of a bookmark ([#114](https://github.com/ZeroAssumptions/aide-de-camp/issues/114))
- **zfs:** Ability to work with bookmarks
- **zfs:** Ability to send snapshot ([#119](https://github.com/ZeroAssumptions/aide-de-camp/issues/119))
- **zfs:** Remove known unknowns from properties ([#121](https://github.com/ZeroAssumptions/aide-de-camp/issues/121))
- Add a single point of logging configuration ([#123](https://github.com/ZeroAssumptions/aide-de-camp/issues/123))

### Styling

- Run cargo fmt ([#117](https://github.com/ZeroAssumptions/aide-de-camp/issues/117))

### Ci

- Add Cirrus-CI ([#76](https://github.com/ZeroAssumptions/aide-de-camp/issues/76))
- Create an memory device for tests on Cirrus ([#77](https://github.com/ZeroAssumptions/aide-de-camp/issues/77))
- **cirrus:** Update FreeBSD builder images to latest production release ([#122](https://github.com/ZeroAssumptions/aide-de-camp/issues/122))

## [0.1.1] - 2019-08-12

### Features

- **zpool:** Add Zpool::add ([#53](https://github.com/ZeroAssumptions/aide-de-camp/issues/53))
- **zpool:** Remove device from zpool ([#60](https://github.com/ZeroAssumptions/aide-de-camp/issues/60))
- **zpool:** Fix parser for logs and caches. Add add_zil and add_cache ([#63](https://github.com/ZeroAssumptions/aide-de-camp/issues/63))
- **zpool:** Add replace_disk. Closes #25 ([#67](https://github.com/ZeroAssumptions/aide-de-camp/issues/67))
- **zpool:** Add regex for another type of vdev reuse. Closes #49 ([#69](https://github.com/ZeroAssumptions/aide-de-camp/issues/69))

### Refactor

- Make Vdev and Zpool structure more understandable ([#39](https://github.com/ZeroAssumptions/aide-de-camp/issues/39))
- Switch to Pairs#as_span ([#56](https://github.com/ZeroAssumptions/aide-de-camp/issues/56))

### Styling

- New fmt config ([#54](https://github.com/ZeroAssumptions/aide-de-camp/issues/54))

### Ci

- Report coverage to Azure Pipelines ([#55](https://github.com/ZeroAssumptions/aide-de-camp/issues/55))
- Try to speedup the build ([#68](https://github.com/ZeroAssumptions/aide-de-camp/issues/68))

<!-- generated by git-cliff -->
