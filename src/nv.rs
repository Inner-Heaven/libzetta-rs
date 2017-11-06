#![deny(missing_docs)]
//! Rust bindings to Name/Value pairs library ([libnv])(https://www.freebsd.org/cgi/man.cgi?query=nv) .
//! It kinda acts like `Map<&str, T>` where T is any type Name/Value list can hold. I honestly
//! don't know if this is used anywhere outside of zfs and that one side-project by someone in google.
//! I only making this in order to work with ZFS, so if you need something that isn't here - PRs
//! welcome.
//! It's missing a few features:
//!     - Sending to socket
//!     - Receving from socket
//!     - Insert/Remove file descriptors
//!     - Insert/Remove binary
//!     - Take operations

use std::convert::{From, Into};
use std::ffi::{CString, CStr, NulError};
use std::slice;
use std::os::unix::io::AsRawFd;

use libc::ENOMEM;

// Importing all because it's cold, I dont want to turn on heater and it's hard to type.
use libnv_sys::*;


quick_error! {
    #[derive(Debug)]
    /// Error kinds for Name/Value library.
    pub enum NvError {
        /// Name a.k.a. key can't contain NULL byte. You going to get this error if you try so.
        InvalidString(err: NulError) {
            from()
        }
        /// error return by ffi. See libc for more information.
        NativeError(code: i32) {}
        /// If trying to set an error on n/v list that already has error
        AlreadySet {}
    }
}

/// Short-cut to Result<T, NvError>.
pub type NvResult<T> = Result<T, NvError>;

/// Enumeration of available data types that the API supports.
#[repr(i32)]
#[derive(Copy, Clone, Debug)]
pub enum NvType {
    /// Empty type
    None = 0,
    /// There is no associated data with the name
    Null = 1,
    /// The value is a `bool` value
    Bool = 2,
    /// The value is a `u64` value
    Number = 3,
    /// The value is a C string
    String = 4,
    /// The value is another `nvlist`
    NvList = 5,
    /// The value is a file descriptor
    Descriptor = 6,
    /// The value is a binary buffer
    Binary = 7,
    /// The value is an array of `bool` values
    BoolArray = 8,
    /// The value is an array of `u64` values
    NumberArray = 9,
    /// The value is an array of C strings
    StringArray = 10,
    /// The value is an array of other `nvlist`'s
    NvListArray = 11,
    /// The value is an array of file descriptors
    DescriptorArray = 12,
}

/// Options available for creation of an `nvlist`
#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NvFlag {
    /// No user specified options.
    None = 0,
    /// Perform case-insensitive lookups of provided names.
    IgnoreCase = 1,
    /// Names in the nvlist do not have to be unique.
    NoUnique = 2,
    /// Allow duplicate case-insensitive keys.
    Both = 3
}

impl From<i32> for NvFlag {
    /// This should be TryFrom. This function WILL panic if you pass incorrect value to it.
    /// However, this should be impossible.
    fn from(source: i32) -> Self {
     match source {
            0 => NvFlag::None,
            1 => NvFlag::IgnoreCase,
            2 => NvFlag::NoUnique,
            3 => NvFlag::Both,
            _ => panic!("Incorrect value passed to NvFlag")
     }
    }
}

macro_rules! impl_list_op {
    ($type_:ty, $method:ident, false) => {
        impl NvTypeOp for $type_ {
            /// Add a `$type_` value to the `NvList`
            fn add_to_list(&self, list: &mut NvList, name: &str) -> NvResult<()> {
                return list.$method(name, *self);
            }
        }
    };
    ($type_:ty, $method:ident, true) => {
        impl NvTypeOp for $type_ {
            /// Add a `$type_` value to the `NvList`
            fn add_to_list(&self, list: &mut NvList, name: &str) -> NvResult<()> {
                return list.$method(name, &*self);
            }
        }
    };
}

/// Marker-ish trait to allow usage of insert method. Implement this for your own types if you don't
/// want to convert to primitive types everytime.
pub trait NvTypeOp {
    /// Add self to given list.
    fn add_to_list(&self, list: &mut NvList, name: &str) -> NvResult<()>;
}

impl_list_op!{bool, insert_bool, false}
impl_list_op!{[bool], insert_bools, true}
impl_list_op!{u8, insert_number, false}
impl_list_op!{u16, insert_number, false}
impl_list_op!{u32, insert_number, false}
impl_list_op!{u64, insert_number, false}
impl_list_op!{[u64], insert_numbers, true}
impl_list_op!{str, insert_string, true}
impl_list_op!{NvList, insert_nvlist, true}

/// If `Some` insert content to the list. If `None` insert null.
impl<T> NvTypeOp for Option<T>
    where T: NvTypeOp {
    fn add_to_list(&self, list: &mut NvList, name: &str) -> NvResult<()> {
        match self {
            &Some(ref val) => val.add_to_list(list, name),
            &None => list.insert_null(name),
        }
    }
}

/// A list of name/value pairs.
#[derive(Debug)]
pub struct NvList {
    ptr: *mut nvlist,
}


#[doc(hidden)]
/// Return new list with no flags.
impl Default for NvList {
    fn default() -> NvList {
        NvList::new(NvFlag::None).expect("Failed to create new list")
    }
}
impl NvList {
    /// Make a copy of a pointer. Danger zone.
    fn as_ptr(&self) -> *mut nvlist {
        self.ptr.clone()
    }
    fn check_if_error(&self) -> NvResult<()> {
        match self.error() {
            errno if errno == 0 => Ok(()),
            errno => Err(NvError::NativeError(errno)),
        }
    }

    /// Create a new name/value pair list (`nvlist`). Call this can only fail when system is out of
    /// memory.
    ///
    /// ```
    /// use libzfs::nv::{NvList, NvFlag};
    ///
    /// let nvlist = NvList::new(NvFlag::None).unwrap();
    /// ```
    pub fn new(flags: NvFlag) -> NvResult<NvList> {
        let raw_list = unsafe { nvlist_create(flags as i32) };
        if raw_list.is_null() {
            Err(NvError::NativeError(ENOMEM))
        } else {
            Ok(NvList { ptr: raw_list })
        }
    }

    /// Determines if the `nvlist` is empty
    ///
    /// ```
    /// use libzfs::nv::{NvList, NvFlag};
    /// let nvlist = NvList::new(NvFlag::IgnoreCase).unwrap();
    /// assert!(nvlist.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        unsafe { nvlist_empty(self.ptr) }
    }

    /// The flags the `nvlist` was created with
    ///
    /// ```
    /// use libzfs::nv::{NvList, NvFlag};
    /// let nvlist = NvList::new(NvFlag::NoUnique).unwrap();
    ///
    /// assert_eq!(nvlist.flags(), NvFlag::NoUnique);
    /// ```
    pub fn flags(&self) -> NvFlag {
        NvFlag::from(unsafe { nvlist_flags(self.ptr) })
    }

    /// Gets error value that the list may have accumulated
    ///
    /// ```
    /// use libzfs::nv::{NvList, NvFlag};
    /// let list = NvList::new(NvFlag::NoUnique).unwrap();
    ///
    /// assert_eq!(0, list.error());
    /// ```
    pub fn error(&self) -> i32 {
        unsafe { nvlist_error(self.ptr) }
    }
    /// Sets the `NvList` to be in an error state
    ///
    /// ```
    /// use libzfs::nv::{NvList, NvFlag};
    ///
    /// let mut list = NvList::new(NvFlag::Both).unwrap();
    ///
    /// // EINVAL
    /// list.set_error(0x16).unwrap();
    ///
    /// assert_eq!(0x16, list.error());
    /// ```
    pub fn set_error(&mut self, error: i32) -> NvResult<()> {
        if self.error() != 0 {
            Err(NvError::AlreadySet)
        } else {
            unsafe { nvlist_set_error(self.ptr, error) }
            Ok(())
        }
    }

    /// Genericially add a single value to the NvList
    ///
    /// ```
    /// use libzfs::nv::{NvList, NvFlag, NvTypeOp};
    ///
    /// let mut list = NvList::default();
    ///
    /// let the_answer: u32 = 1776;
    /// let not_the_answer: Option<u64> = None;
    ///
    /// list.insert("Important year", the_answer);
    /// list.insert("not important year", not_the_answer);
    /// let copy = list.clone();
    /// list.insert("foo", copy);
    ///
    /// assert_eq!(list.get_number("Important year").unwrap().unwrap(), 1776);
    /// ```
    pub fn insert<T: NvTypeOp>(&mut self, name: &str, value: T) -> NvResult<()> {
        value.add_to_list(self, name)
    }

    /// Add a null value to the `NvList`
    ///
    /// ```
    /// use libzfs::nv::{NvList, NvFlag, NvTypeOp};
    /// let mut list = NvList::new(NvFlag::Both).unwrap();
    /// list.insert_null("Hello, World!");
    /// ```
    pub fn insert_null(&mut self, name: &str) -> NvResult<()> {
        let c_name = CString::new(name)?;
        unsafe {
            nvlist_add_null(self.ptr, c_name.as_ptr());
        }
        self.check_if_error()
    }
    /// Add a number to the `NvList`. Number will be converted into u64. 
    ///
    /// ```
    /// use libzfs::nv::{NvList, NvFlag};
    ///
    /// let mut list = NvList::new(NvFlag::Both).unwrap();
    ///
    /// list.insert_number("Important year", 1776u64);
    /// ```
    pub fn insert_number<I: Into<u64>>(&mut self, name: &str, value: I) -> NvResult<()> {
        let c_name = CString::new(name)?;
        unsafe {
            nvlist_add_number(self.ptr, c_name.as_ptr(), value.into());
        }
        self.check_if_error()
    }
    /// Add a `bool` to the list
    pub fn insert_bool(&mut self, name: &str, value: bool) -> NvResult<()> {
        let c_name = CString::new(name)?;
        unsafe {
            nvlist_add_bool(self.ptr, c_name.as_ptr(), value);
        }
        self.check_if_error()
    }

     /// Add string to the list
    pub fn insert_string(&mut self, name: &str, value: &str) -> NvResult<()> {
        let c_name = CString::new(name)?;
        let c_value = CString::new(value)?;
        unsafe {
            nvlist_add_string(self.ptr, c_name.as_ptr(), c_value.as_ptr());
        }
        self.check_if_error()
    }

    /// Add `NvList` to the list
    ///
    /// ```
    /// use libzfs::nv::{NvList, NvFlag};
    ///
    /// let mut list = NvList::default();
    ///
    /// let other_list = NvList::default();
    ///
    /// list.insert_nvlist("other list", &other_list).unwrap();
    ///
    /// ```
    pub fn insert_nvlist(&mut self, name: &str, value: &NvList) -> NvResult<()> {
        let c_name = CString::new(name)?;
        if !value.as_ptr().is_null() {
            unsafe {
                nvlist_add_nvlist(self.ptr, c_name.as_ptr(), value.as_ptr());
            }
        }
        self.check_if_error()
    }

    /// Add binary data to the list. TODO: make this safe.
    pub unsafe fn add_binary(&mut self, name: &str, value: *mut i8, size: u32) -> NvResult<()> {
        let c_name = CString::new(name)?;
        nvlist_add_binary(self.ptr, c_name.as_ptr(), value, size);
        self.check_if_error()
    }

    /// Add an array of `bool` values
    ///
    /// ```
    /// use libzfs::nv::{NvList, NvFlag};
    ///
    /// let mut list = NvList::new(NvFlag::Both).unwrap();
    ///
    /// let slice = [true, false, true, false];
    ///
    /// list.insert_bools("Important year", &slice);
    /// ```
    pub fn insert_bools(&mut self, name: &str, value: &[bool]) -> NvResult<()> {
        let c_name = CString::new(name)?;
        unsafe {
            nvlist_add_bool_array(self.ptr, c_name.as_ptr(), value.as_ptr(), value.len());
        }
        self.check_if_error()
    }

    /// Add an array if `u64`. TODO: Make it work with any number...
    ///
    /// ```
    /// use libzfs::nv::{NvList, NvFlag};
    ///
    /// let mut list = NvList::new(NvFlag::None).unwrap();
    ///
    /// let slice = [1776, 2017];
    ///
    /// list.insert_numbers("Important year", &slice);
    ///
    /// ```
    pub fn insert_numbers(&mut self, name: &str, value: &[u64]) -> NvResult<()> {
        let c_name = CString::new(name)?;
        unsafe {
            nvlist_add_number_array(self.ptr, c_name.as_ptr(), value.as_ptr(), value.len());
        }
        self.check_if_error()
    }

    /// Add an array of strings
    ///
    /// ```
    /// use libzfs::nv::{NvList, NvFlag};
    ///
    /// let mut list = NvList::new(NvFlag::None).unwrap();
    ///
    /// let orig = ["Hello", "World!"];
    ///
    /// list.insert_strings("key", &orig).unwrap();
    ///
    /// let vec = list.get_strings("key").unwrap().unwrap();
    ///
    /// assert_eq!(*vec, ["Hello", "World!"]);
    /// ```
    pub fn insert_strings(&mut self, name: &str, value: &[&str]) -> NvResult<()> {
        let c_name = CString::new(name)?;
        let strings: Vec<CString> = value.iter()
            .map(|e| CString::new(*e))
            .map(|e| e.expect("Failed to convert str to Cstring"))
            .collect();
        unsafe {
            let pointers: Vec<*const i8> = strings.iter()
                .map(|e| e.as_ptr())
                .collect();

            nvlist_add_string_array(self.ptr,
                                    c_name.as_ptr(),
                                    pointers.as_slice().as_ptr(),
                                    strings.len());
        }
        self.check_if_error()
    }

    /// Add an array of `NvList`s
    ///
    /// ```
    /// use libzfs::nv::{NvList, NvFlag};
    ///
    /// let mut list = NvList::new(NvFlag::Both).unwrap();
    ///
    /// let slice = [NvList::new(NvFlag::Both).unwrap(), NvList::new(NvFlag::Both).unwrap(),
    ///              NvList::new(NvFlag::None).unwrap()];
    ///
    /// list.insert_nvlists("nvlists", &slice);
    ///
    /// let mut nvlists = list.get_nvlists("nvlists").unwrap().unwrap();
    ///
    /// assert_eq!(NvFlag::None, nvlists.pop().unwrap().flags());
    /// ```
    pub fn insert_nvlists(&mut self, name: &str, value: &[NvList]) -> NvResult<()> {
        let c_name = CString::new(name)?;
        let vec = value.to_vec();
        unsafe {
            let lists: Vec<*const nvlist> = vec.iter()
                .map(|item| item.as_ptr() as *const nvlist)
                .collect();
            nvlist_add_nvlist_array(self.ptr, c_name.as_ptr(), lists.as_slice().as_ptr(), lists.len());
        }
        self.check_if_error()
    }

    /// Returns `true` if a name/value pair exists in the `NvList` and `false` otherwise.
    ///
    /// ```
    /// use libzfs::nv::{NvList, NvFlag};
    ///
    /// let mut list = NvList::new(NvFlag::Both).unwrap();
    ///
    /// let result = list.insert_number("Important year", 1776u64);
    /// assert!(result.is_ok());
    ///
    /// assert!(list.contains_key("Important year").unwrap());
    /// ```
    pub fn contains_key(&self, name: &str) -> NvResult<bool> {
        let c_name = CString::new(name)?;
        unsafe { Ok(nvlist_exists(self.ptr, c_name.as_ptr())) }
    }

    /// Returns `true` if a name/value pair of the specified type exists in the `NvList` and `false` otherwise
    /// ```
    /// use libzfs::nv::{NvList, NvFlag};
    ///
    /// let mut list = NvList::new(NvFlag::Both).unwrap();
    ///
    /// let result = list.insert_number("Important year", 1776u64);
    /// assert!(result.is_ok());
    ///
    /// assert!(!list.contains_key_with_type("Important year", NvType::Bool));
    /// ```
    pub fn contains_key_with_type(&self, name: &str, ty: NvType) -> NvResult<bool> {
        let c_name = CString::new(name)?;
        unsafe { Ok(nvlist_exists_type(self.ptr, c_name.as_ptr(), ty as i32)) }
    }


    /// Get the first matching `bool` value paired with
    /// the given name
    ///
    /// ```
    /// use libzfs::nv::NvList;
    ///
    /// let mut list = NvList::default();
    ///
    /// list.insert_bool("Did history start on 1776?", true).unwrap();
    ///
    /// assert!(list.get_bool("Did history start on 1776?").unwrap().unwrap(), true);
    /// ```
    pub fn get_bool(&self, name: &str) -> NvResult<Option<bool>> {
        let c_name = CString::new(name)?;
        unsafe {
            if nvlist_exists_bool(self.ptr, c_name.as_ptr()) {
                Ok(Some(nvlist_get_bool(self.ptr, c_name.as_ptr())))
            } else {
                Ok(None)
            }
        }
    }

    /// Get the first matching `u64` value paired with
    /// the given name
    pub fn get_number(&self, name: &str) -> NvResult<Option<u64>> {
        let c_name = CString::new(name)?;
        unsafe {
            if nvlist_exists_number(self.ptr, c_name.as_ptr()) {
                Ok(Some(nvlist_get_number(self.ptr, c_name.as_ptr())))
            } else {
                Ok(None)
            }
        }
    }

    /// Get the first matching `u64` value paired with
    /// the given name
    ///
    /// ```
    /// use libzfs::nv::{NvList, NvFlag};
    ///
    /// // Note: we're allowing duplicate values per name
    /// let mut list = NvList::default();
    ///
    /// list.insert_string("Hello", "World!").unwrap();
    ///
    /// assert_eq!(list.get_string("Hello").unwrap().unwrap(), "World!");
    /// ```
    pub fn get_string(&self, name: &str) -> NvResult<Option<String>> {
        let c_name = CString::new(name)?;
        unsafe {
            if nvlist_exists_string(self.ptr, c_name.as_ptr()) {
                let ret = nvlist_get_string(self.ptr, c_name.as_ptr());
                if ret.is_null() {
                    Ok(None)
                } else {
                    let len = strlen(ret);
                    Ok(Some(String::from_raw_parts(ret as *mut u8, len, len)))
                }
            } else {
                Ok(None)
            }

        }
    }

    /// Get the first matching `NvList` value paired with
    /// the given name and clone it
    ///
    /// ```
    /// use libzfs::nv::{NvList, NvFlag};
    ///
    /// let mut list = NvList::new(NvFlag::Both).unwrap();
    ///
    /// list.insert_bool("other list", true).unwrap();
    ///
    /// let mut other_list = NvList::new(NvFlag::None).unwrap();
    /// other_list.insert_number("Important year", 42u32).unwrap();
    ///
    /// list.insert_nvlist("other list", &other_list).unwrap();
    ///
    /// // Since we use `get_nvlist` we will get the
    /// // NvList not the boolean value
    /// let other_nvlist = list.get_nvlist("other list").unwrap().unwrap();
    ///
    /// assert_eq!(other_nvlist.get_number("Important year").unwrap().unwrap(), 42);
    /// ```
    pub fn get_nvlist(&self, name: &str) -> NvResult<Option<NvList>> {
        let c_name = CString::new(name)?;
        unsafe {
            if nvlist_exists_nvlist(self.ptr, c_name.as_ptr()) {
                let res = nvlist_get_nvlist(self.ptr, c_name.as_ptr());
                Ok(Some(NvList { ptr: nvlist_clone(res) }))
            } else {
                Ok(None)
            }
        }
    }


    /// Get a `&[bool]` from the `NvList`
    ///
    /// ```
    /// use libzfs::nv::{NvList, NvFlag};
    ///
    /// // Note: we're allowing duplicate values per name
    /// let mut list = NvList::new(NvFlag::None).unwrap();
    ///
    /// list.insert_bools("true/false", &[true, false, true]).unwrap();
    ///
    /// assert_eq!(list.get_bools("true/false").unwrap().unwrap(), &[true, false, true]);
    /// ```
    pub fn get_bools<'a>(&'a self, name: &str) -> NvResult<Option<&'a [bool]>> {
        let c_name = CString::new(name)?;
        unsafe {
            if nvlist_exists_bool_array(self.ptr, c_name.as_ptr()) {
                let mut len: usize = 0;
                let arr = nvlist_get_bool_array(self.ptr, c_name.as_ptr(), &mut len as *mut usize);
                Ok(Some(slice::from_raw_parts(arr as *const bool, len)))
            } else {
                Ok(None)
            }
        }
    }

    /// Get a `&[u64]` slice from the `NvList`
    ///
    /// ```
    /// use libzfs::nv::{NvList, NvFlag};
    ///
    /// // Note: we're allowing duplicate values per name
    /// let mut list = NvList::default();
    ///
    /// list.insert_numbers("The Year", &[1, 7, 7, 6]).unwrap();
    ///
    /// assert_eq!(list.get_numbers("The Year").unwrap().unwrap(), &[1, 7, 7, 6]);
    /// ```
    pub fn get_numbers<'a>(&'a self, name: &str) -> NvResult<Option<&'a [u64]>> {
        let c_name = CString::new(name)?;
        unsafe {
            if nvlist_exists_number_array(self.ptr, c_name.as_ptr()) {
                let mut len: usize = 0;
                let arr =
                    nvlist_get_number_array(self.ptr, c_name.as_ptr(), &mut len as *mut usize);
                Ok(Some(slice::from_raw_parts(arr as *const u64, len)))
            } else {
                Ok(None)
            }
        }
    }

    /// Get a `Vec<String>` of the first string slice added to the `NvList`
    /// for the given name
    ///
    pub fn get_strings(&self, name: &str) -> NvResult<Option<Vec<String>>> {
        let c_name = CString::new(name)?;
        unsafe {
            if nvlist_exists_string_array(self.ptr, c_name.as_ptr()) {
                let mut len: usize = 0;
                let arr = nvlist_get_string_array(self.ptr, c_name.as_ptr(), &mut len as *mut usize);
                let slice = slice::from_raw_parts(arr as *const *const i8, len);
                let strings = slice.iter()
                    .map(|ptr| *ptr)
                    .map(|ptr| CStr::from_ptr(ptr))
                    .map(|cstr| cstr.to_string_lossy())
                    .map(|string| String::from(string))
                    .collect();
                Ok(Some(strings))
            } else {
                Ok(None)
            }
        }
    }

    /// Get an array of `NvList`.
    ///
    /// ```
    /// use libzfs::nv::{NvList, NvFlag};
    ///
    /// let mut list = NvList::new(NvFlag::None).unwrap();
    ///
    /// list.insert_nvlists("lists", &[NvList::default(),
    ///                                       NvList::default()]).unwrap();
    ///
    /// let vec = list.get_nvlists("lists").unwrap().unwrap();
    ///
    /// assert_eq!(vec.len(), 2);
    /// assert_eq!(vec[0].flags(), NvFlag::None);
    /// ```
    pub fn get_nvlists(&self, name: &str) -> NvResult<Option<Vec<NvList>>> {
        let c_name = CString::new(name)?;
        unsafe {
            if nvlist_exists_nvlist_array(self.ptr, c_name.as_ptr()) {
                let mut len: usize = 0;
                let arr =
                    nvlist_get_nvlist_array(self.ptr, c_name.as_ptr(), &mut len as *mut usize);
                let slice = slice::from_raw_parts(arr as *const *const nvlist, len);
                Ok(Some(slice.iter()
                     .map(|item| NvList { ptr: nvlist_clone(*item) })
                     .collect()))
            } else {
                Ok(None)
            }
        }
    }

    /// Write `NvList` to a file descriptor.
    ///
    /// ```
    /// use std::fs::File;
    /// use libzfs::nv::NvList;
    ///
    /// let mut list = NvList::default();
    ///
    /// list.insert_number("Important year", 1776u64);
    ///
    /// list.dump(File::create("/tmp/libzfs_nv.dump").unwrap());
    /// ```
    pub fn dump<T: AsRawFd>(&self, out: T) -> NvResult<()> {
        unsafe { nvlist_dump(self.ptr, out.as_raw_fd()) }
        self.check_if_error()
    }

    /// The size of the current list
    pub fn len(&self) -> i32 {
        unsafe { nvlist_size(self.ptr) }
    }

    /// Removes a key from the `NvList`.
    pub fn remove(&mut self, name: &str) -> NvResult<()> {
        let c_name = CString::new(name)?;
        unsafe {
            nvlist_free(self.ptr, c_name.as_ptr());
        }
        self.check_if_error()
    }

    /// Remove the element of the given name and type
    /// from the `NvList`
    pub fn remove_with_type(&mut self, name: &str, ty: NvType) -> NvResult<()> {
        let c_name = CString::new(name)?;
        unsafe {
            nvlist_free_type(self.ptr, c_name.as_ptr(), ty as i32);
        }
        self.check_if_error()
    }
}

impl Clone for NvList {
    /// Clone list using libnv method. This will perform deep copy.
    fn clone(&self) -> NvList {
        NvList { ptr: unsafe { nvlist_clone(self.ptr) } }
    }
}


impl Drop for NvList {
    /// Using libnv method.
    fn drop(&mut self) {
        unsafe { nvlist_destroy(self.ptr); }
    }
}
