#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[allow(missing_docs)]
pub enum nvlist {}

extern "C" {
    pub fn nvlist_create(flags: i32) -> *mut nvlist;
    pub fn nvlist_destroy(list: *mut nvlist) -> ();
    pub fn nvlist_empty(list: *const nvlist) -> bool;
    pub fn nvlist_flags(list: *const nvlist) -> i32;
    pub fn nvlist_error(list: *const nvlist) -> i32;
    pub fn nvlist_set_error(list: *mut nvlist, error: i32) -> ();
    pub fn nvlist_clone(list: *const nvlist) -> *mut nvlist;
    pub fn nvlist_dump(list: *const nvlist, fd: i32) -> ();
    pub fn nvlist_size(list: *const nvlist) -> i32;
    // add value
    pub fn nvlist_add_null(list: *mut nvlist, name: *const i8) -> ();
    pub fn nvlist_add_bool(list: *mut nvlist, name: *const i8, value: bool) -> ();
    pub fn nvlist_add_number(list: *mut nvlist, name: *const i8, value: u64) -> ();
    pub fn nvlist_add_string(list: *mut nvlist, name: *const i8, value: *const i8) -> ();
    pub fn nvlist_add_nvlist(list: *mut nvlist, name: *const i8, value: *const nvlist) -> ();
    pub fn nvlist_add_binary(list: *mut nvlist, name: *const i8, value: *mut i8, size: u32) -> ();
    pub fn nvlist_add_bool_array(list: *mut nvlist,
                             name: *const i8,
                             value: *const bool,
                             size: usize)
                             -> ();
    pub fn nvlist_add_number_array(list: *mut nvlist,
                               name: *const i8,
                               value: *const u64,
                               size: usize)
                               -> ();
    pub fn nvlist_add_string_array(list: *mut nvlist,
                               name: *const i8,
                               value: *const *const i8,
                               size: usize)
                               -> ();
    pub fn nvlist_add_nvlist_array(list: *mut nvlist,
                               name: *const i8,
                               value: *const *const nvlist,
                               size: usize)
                               -> ();
    pub fn nvlist_exists(list: *const nvlist, name: *const i8) -> bool;
    pub fn nvlist_exists_type(list: *const nvlist, name: *const i8, ty: i32) -> bool;
    pub fn nvlist_exists_bool(list: *const nvlist, name: *const i8) -> bool;
    pub fn nvlist_exists_number(list: *const nvlist, name: *const i8) -> bool;
    pub fn nvlist_exists_string(list: *const nvlist, name: *const i8) -> bool;
    pub fn nvlist_exists_nvlist(list: *const nvlist, name: *const i8) -> bool;
    pub fn nvlist_exists_bool_array(list: *const nvlist, name: *const i8) -> bool;
    pub fn nvlist_exists_number_array(list: *const nvlist, name: *const i8) -> bool;
    pub fn nvlist_exists_string_array(list: *const nvlist, name: *const i8) -> bool;
    pub fn nvlist_exists_nvlist_array(list: *const nvlist, name: *const i8) -> bool;
    pub fn nvlist_get_bool(list: *const nvlist, name: *const i8) -> bool;
    pub fn nvlist_get_number(list: *const nvlist, name: *const i8) -> u64;
    pub fn nvlist_get_string(list: *const nvlist, name: *const i8) -> *const i8;
    pub fn nvlist_get_nvlist(list: *const nvlist, name: *const i8) -> *const nvlist;
    pub fn nvlist_get_bool_array(list: *const nvlist, name: *const i8, len: *const usize) -> *mut bool;
    pub fn nvlist_get_number_array(list: *const nvlist,
                               name: *const i8,
                               len: *const usize)
                               -> *mut u64;
    pub fn nvlist_get_string_array(list: *const nvlist,
                               name: *const i8,
                               len: *const usize)
                               -> *mut *mut i8;
    pub fn nvlist_get_nvlist_array(list: *const nvlist,
                               name: *const i8,
                               len: *const usize)
                               -> *mut *mut nvlist;
    pub fn nvlist_free(list: *mut nvlist, name: *const i8) -> ();
    pub fn nvlist_free_type(list: *mut nvlist, name: *const i8, ty: i32) -> ();
    pub fn strlen(target: *const i8) -> usize;
}
