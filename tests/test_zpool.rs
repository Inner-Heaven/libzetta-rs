extern crate libzfs;

use libzfs::zpool::{ZpoolEngine, ZpoolOpen3};


#[test]
fn just_for_lulz() {
    let engine = ZpoolOpen3::default();

    assert!(engine.exists("z").unwrap());
}
