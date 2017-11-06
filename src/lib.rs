extern crate libc;
extern crate libnv_sys;
#[macro_use]
extern crate quick_error;

pub mod nv;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
