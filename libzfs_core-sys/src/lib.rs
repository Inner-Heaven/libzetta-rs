#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate libnv_sys;

use libnv_sys::nvlist as nvlist_t;


pub type boolean_t = bool;

pub const lzc_send_flags_LZC_SEND_FLAG_EMBED_DATA: lzc_send_flags = 1;
pub const lzc_send_flags_LZC_SEND_FLAG_LARGE_BLOCK: lzc_send_flags = 2;

pub type lzc_send_flags = ::std::os::raw::c_uint;

pub const dmu_objset_type_t_DMU_OST_NONE: dmu_objset_type_t = 0;
pub const dmu_objset_type_t_DMU_OST_META: dmu_objset_type_t = 1;
pub const dmu_objset_type_t_DMU_OST_ZFS: dmu_objset_type_t = 2;
pub const dmu_objset_type_t_DMU_OST_ZVOL: dmu_objset_type_t = 3;
pub const dmu_objset_type_t_DMU_OST_OTHER: dmu_objset_type_t = 4;
pub const dmu_objset_type_t_DMU_OST_ANY: dmu_objset_type_t = 5;
pub const dmu_objset_type_t_DMU_OST_NUMTYPES: dmu_objset_type_t = 6;

pub type dmu_objset_type_t = ::std::os::raw::c_uint;

extern "C" {
    pub fn libzfs_core_init() -> ::std::os::raw::c_int;
    pub fn libzfs_core_fini();
    pub fn lzc_snapshot(arg1: *mut nvlist_t,
                        arg2: *mut nvlist_t,
                        arg3: *mut *mut nvlist_t)
                        -> ::std::os::raw::c_int;
    pub fn lzc_create(arg1: *const ::std::os::raw::c_char,
                      arg2: dmu_objset_type_t,
                      arg3: *mut nvlist_t)
                      -> ::std::os::raw::c_int;
    pub fn lzc_clone(arg1: *const ::std::os::raw::c_char,
                     arg2: *const ::std::os::raw::c_char,
                     arg3: *mut nvlist_t)
                     -> ::std::os::raw::c_int;
    pub fn lzc_destroy_snaps(arg1: *mut nvlist_t,
                             arg2: boolean_t,
                             arg3: *mut *mut nvlist_t)
                             -> ::std::os::raw::c_int;
    pub fn lzc_bookmark(arg1: *mut nvlist_t, arg2: *mut *mut nvlist_t) -> ::std::os::raw::c_int;
    pub fn lzc_get_bookmarks(arg1: *const ::std::os::raw::c_char,
                             arg2: *mut nvlist_t,
                             arg3: *mut *mut nvlist_t)
                             -> ::std::os::raw::c_int;
    pub fn lzc_destroy_bookmarks(arg1: *mut nvlist_t,
                                 arg2: *mut *mut nvlist_t)
                                 -> ::std::os::raw::c_int;
    pub fn lzc_snaprange_space(arg1: *const ::std::os::raw::c_char,
                               arg2: *const ::std::os::raw::c_char,
                               arg3: *mut u64)
                               -> ::std::os::raw::c_int;
    pub fn lzc_hold(arg1: *mut nvlist_t,
                    arg2: ::std::os::raw::c_int,
                    arg3: *mut *mut nvlist_t)
                    -> ::std::os::raw::c_int;
    pub fn lzc_release(arg1: *mut nvlist_t, arg2: *mut *mut nvlist_t) -> ::std::os::raw::c_int;
    pub fn lzc_get_holds(arg1: *const ::std::os::raw::c_char,
                         arg2: *mut *mut nvlist_t)
                         -> ::std::os::raw::c_int;
    pub fn lzc_send(arg1: *const ::std::os::raw::c_char,
                    arg2: *const ::std::os::raw::c_char,
                    arg3: ::std::os::raw::c_int,
                    arg4: lzc_send_flags)
                    -> ::std::os::raw::c_int;
    pub fn lzc_receive(arg1: *const ::std::os::raw::c_char,
                       arg2: *mut nvlist_t,
                       arg3: *const ::std::os::raw::c_char,
                       arg4: boolean_t,
                       arg5: ::std::os::raw::c_int)
                       -> ::std::os::raw::c_int;
    pub fn lzc_send_space(arg1: *const ::std::os::raw::c_char,
                          arg2: *const ::std::os::raw::c_char,
                          arg3: *mut u64)
                          -> ::std::os::raw::c_int;
    pub fn lzc_exists(arg1: *const ::std::os::raw::c_char) -> boolean_t;
    pub fn lzc_rollback(arg1: *const ::std::os::raw::c_char,
                        arg2: *mut ::std::os::raw::c_char,
                        arg3: ::std::os::raw::c_int)
                        -> ::std::os::raw::c_int;
    pub fn lzc_promote(arg1: *const ::std::os::raw::c_char,
                       arg2: *mut nvlist_t,
                       arg3: *mut *mut nvlist_t)
                       -> ::std::os::raw::c_int;
    pub fn lzc_rename(arg1: *const ::std::os::raw::c_char,
                      arg2: *const ::std::os::raw::c_char,
                      arg3: *mut nvlist_t,
                      arg4: *mut *mut ::std::os::raw::c_char)
                      -> ::std::os::raw::c_int;
    pub fn lzc_destroy_one(fsname: *const ::std::os::raw::c_char,
                           arg1: *mut nvlist_t)
                           -> ::std::os::raw::c_int;
    pub fn lzc_inherit(fsname: *const ::std::os::raw::c_char,
                       name: *const ::std::os::raw::c_char,
                       arg1: *mut nvlist_t)
                       -> ::std::os::raw::c_int;
    pub fn lzc_set_props(arg1: *const ::std::os::raw::c_char,
                         arg2: *mut nvlist_t,
                         arg3: *mut nvlist_t,
                         arg4: *mut nvlist_t)
                         -> ::std::os::raw::c_int;
    pub fn lzc_list(arg1: *const ::std::os::raw::c_char,
                    arg2: *mut nvlist_t)
                    -> ::std::os::raw::c_int;
}
