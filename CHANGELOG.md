#  (2020-03-01)

# [v0.2.0](https://github.com/Inner-Heaven/libzetta-rs/compare/v0.1.2...v0.2.0) (2020-03-01)


### Bug Fixes

* **zpool:** Integer overflow in zpool parser ([dd0da1b](https://github.com/Inner-Heaven/libzetta-rs/commit/dd0da1b)), closes [#88](https://github.com/Inner-Heaven/libzetta-rs/issues/88)


### Features

* **zfs:** Ability to read filesystem dataset properties ([#111](https://github.com/Inner-Heaven/libzetta-rs/issues/111)) ([1f89fc3](https://github.com/Inner-Heaven/libzetta-rs/commit/1f89fc3))
* **zfs:** Ability to send snapshot ([#119](https://github.com/Inner-Heaven/libzetta-rs/issues/119)) ([1ac0560](https://github.com/Inner-Heaven/libzetta-rs/commit/1ac0560))
* **zfs:** Ability to work with bookmarks ([7721776](https://github.com/Inner-Heaven/libzetta-rs/commit/7721776))
* **zfs:** Add PathExt trait to make it easier to work with dataset names ([#100](https://github.com/Inner-Heaven/libzetta-rs/issues/100)) ([ffdaf4d](https://github.com/Inner-Heaven/libzetta-rs/commit/ffdaf4d))
* **zfs:** Basic zfs create and destroy operations ([#91](https://github.com/Inner-Heaven/libzetta-rs/issues/91)) ([fdb40c5](https://github.com/Inner-Heaven/libzetta-rs/commit/fdb40c5))
* **zfs:** Check existence of dataset. ([bc0b632](https://github.com/Inner-Heaven/libzetta-rs/commit/bc0b632))
* **zfs:** Listing filesystems and volumes ([#94](https://github.com/Inner-Heaven/libzetta-rs/issues/94)) ([978645c](https://github.com/Inner-Heaven/libzetta-rs/commit/978645c))
* **zfs:** Pass errors from lzc snapshot call to the consumer. ([#102](https://github.com/Inner-Heaven/libzetta-rs/issues/102)) ([f0bcbbd](https://github.com/Inner-Heaven/libzetta-rs/commit/f0bcbbd)), closes [#99](https://github.com/Inner-Heaven/libzetta-rs/issues/99)
* **zfs:** Read properties of a bookmark ([#114](https://github.com/Inner-Heaven/libzetta-rs/issues/114)) ([88ea5f0](https://github.com/Inner-Heaven/libzetta-rs/commit/88ea5f0))
* **zfs:** Read properties of a snapshot ([#112](https://github.com/Inner-Heaven/libzetta-rs/issues/112)) ([1348a0f](https://github.com/Inner-Heaven/libzetta-rs/commit/1348a0f))
* **zfs:** Read properties of a volume ([#113](https://github.com/Inner-Heaven/libzetta-rs/issues/113)) ([4361251](https://github.com/Inner-Heaven/libzetta-rs/commit/4361251))
* **zfs:** Remove known unknowns from properties ([#121](https://github.com/Inner-Heaven/libzetta-rs/issues/121)) ([2ba858c](https://github.com/Inner-Heaven/libzetta-rs/commit/2ba858c))
* Add a single point of logging configuration ([#123](https://github.com/Inner-Heaven/libzetta-rs/issues/123)) ([3f4bba0](https://github.com/Inner-Heaven/libzetta-rs/commit/3f4bba0))
* Fuzzy testing target ([#90](https://github.com/Inner-Heaven/libzetta-rs/issues/90)) ([8c300ff](https://github.com/Inner-Heaven/libzetta-rs/commit/8c300ff))
* Inception of ZFS module ([#83](https://github.com/Inner-Heaven/libzetta-rs/issues/83)) ([ce626a0](https://github.com/Inner-Heaven/libzetta-rs/commit/ce626a0))
* remove unicode feature from regex crate ([#93](https://github.com/Inner-Heaven/libzetta-rs/issues/93)) ([e175499](https://github.com/Inner-Heaven/libzetta-rs/commit/e175499))


## [0.1.1](https://github.com/Inner-Heaven/libzetta-rs/compare/2cbf197...v0.1.1) (2019-08-12)


### Features

* **zpool:** Add regex for another type of vdev reuse. Closes [#49](https://github.com/Inner-Heaven/libzetta-rs/issues/49) ([#69](https://github.com/Inner-Heaven/libzetta-rs/issues/69)) ([b7b0466](https://github.com/Inner-Heaven/libzetta-rs/commit/b7b0466))
* **zpool:** Add replace_disk. Closes [#25](https://github.com/Inner-Heaven/libzetta-rs/issues/25) ([#67](https://github.com/Inner-Heaven/libzetta-rs/issues/67)) ([c4cfc40](https://github.com/Inner-Heaven/libzetta-rs/commit/c4cfc40))
* **zpool:** Add Zpool::add ([#53](https://github.com/Inner-Heaven/libzetta-rs/issues/53)) ([2cbf197](https://github.com/Inner-Heaven/libzetta-rs/commit/2cbf197))
* **zpool:** Fix parser for logs and caches. Add add_zil and add_cache ([#63](https://github.com/Inner-Heaven/libzetta-rs/issues/63)) ([ff2eda5](https://github.com/Inner-Heaven/libzetta-rs/commit/ff2eda5)), closes [#62](https://github.com/Inner-Heaven/libzetta-rs/issues/62) [#61](https://github.com/Inner-Heaven/libzetta-rs/issues/61)
* **zpool:** Remove device from zpool ([#60](https://github.com/Inner-Heaven/libzetta-rs/issues/60)) ([28c90ff](https://github.com/Inner-Heaven/libzetta-rs/commit/28c90ff)), closes [#59](https://github.com/Inner-Heaven/libzetta-rs/issues/59)
