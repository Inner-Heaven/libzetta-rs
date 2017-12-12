#![feature(test)]
extern crate libzfs;
use libzfs::zpool::ZpoolProperties;
extern crate test;
use test::Bencher;

#[bench]
fn zpool_param_parsing(b: &mut Bencher) {

    let line = test::black_box(b"69120\t0\t-\t1.00x\t-\t1%\t67039744\t0\t15867762423891129245\tONLINE\t67108864\t0\t-\toff\toff\toff\t-\t-\t0\ton\twait\n");
    b.iter(|| { let props = ZpoolProperties::try_from_stdout(line).unwrap(); })
}
