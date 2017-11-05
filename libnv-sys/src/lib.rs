#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[allow(missing_docs)]
pub enum nvlist {}

extern "C" {
    fn nvlist_create(flags: i32) -> *mut nvlist;
    fn nvlist_destroy(list: *mut nvlist) -> ();
    fn nvlist_empty(list: *const nvlist) -> bool;
    fn nvlist_flags(list: *const nvlist) -> i32;
    fn nvlist_error(list: *const nvlist) -> i32;
    fn nvlist_set_error(list: *mut nvlist, error: i32) -> ();
    fn nvlist_clone(list: *const nvlist) -> *mut nvlist;
    fn nvlist_dump(list: *const nvlist, fd: i32) -> ();
    fn nvlist_size(list: *const nvlist) -> i32;
    // add value
    fn nvlist_add_null(list: *mut nvlist, name: *const i8) -> ();
    fn nvlist_add_bool(list: *mut nvlist, name: *const i8, value: bool) -> ();
    fn nvlist_add_number(list: *mut nvlist, name: *const i8, value: u64) -> ();
    fn nvlist_add_string(list: *mut nvlist, name: *const i8, value: *const i8) -> ();
    fn nvlist_add_nvlist(list: *mut nvlist, name: *const i8, value: *const nvlist) -> ();
    fn nvlist_add_binary(list: *mut nvlist, name: *const i8, value: *mut i8, size: u32) -> ();
    fn nvlist_add_bool_array(list: *mut nvlist,
                             name: *const i8,
                             value: *const bool,
                             size: usize)
                             -> ();
    fn nvlist_add_number_array(list: *mut nvlist,
                               name: *const i8,
                               value: *const u64,
                               size: usize)
                               -> ();
    fn nvlist_add_string_array(list: *mut nvlist,
                               name: *const i8,
                               value: *const *const i8,
                               size: usize)
                               -> ();
    fn nvlist_add_nvlist_array(list: *mut nvlist,
                               name: *const i8,
                               value: *const *const nvlist,
                               size: usize)
                               -> ();
    fn nvlist_exists(list: *const nvlist, name: *const i8) -> bool;
    fn nvlist_exists_type(list: *const nvlist, name: *const i8, ty: i32) -> bool;
    fn nvlist_exists_bool(list: *const nvlist, name: *const i8) -> bool;
    fn nvlist_exists_number(list: *const nvlist, name: *const i8) -> bool;
    fn nvlist_exists_string(list: *const nvlist, name: *const i8) -> bool;
    fn nvlist_exists_nvlist(list: *const nvlist, name: *const i8) -> bool;
    fn nvlist_exists_bool_array(list: *const nvlist, name: *const i8) -> bool;
    fn nvlist_exists_number_array(list: *const nvlist, name: *const i8) -> bool;
    fn nvlist_exists_string_array(list: *const nvlist, name: *const i8) -> bool;
    fn nvlist_exists_nvlist_array(list: *const nvlist, name: *const i8) -> bool;
    fn nvlist_get_bool(list: *const nvlist, name: *const i8) -> bool;
    fn nvlist_get_number(list: *const nvlist, name: *const i8) -> u64;
    fn nvlist_get_string(list: *const nvlist, name: *const i8) -> *const i8;
    fn nvlist_get_nvlist(list: *const nvlist, name: *const i8) -> *const nvlist;
    fn nvlist_get_bool_array(list: *const nvlist, name: *const i8, len: *const usize) -> *mut bool;
    fn nvlist_get_number_array(list: *const nvlist,
                               name: *const i8,
                               len: *const usize)
                               -> *mut u64;
    fn nvlist_get_string_array(list: *const nvlist,
                               name: *const i8,
                               len: *const usize)
                               -> *const *const i8;
    fn nvlist_get_nvlist_array(list: *const nvlist,
                               name: *const i8,
                               len: *const usize)
                               -> *const *const nvlist;
    fn nvlist_free(list: *mut nvlist, name: *const i8) -> ();
    fn nvlist_free_type(list: *mut nvlist, name: *const i8, ty: i32) -> ();
    fn strlen(target: *const i8) -> usize;
}
