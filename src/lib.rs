use std::error::Error;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};

// #[link(name = "libc")]
extern "system" {
    #[link_name = "dlopen"]
    fn _dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    #[link_name = "dlclose"]
    fn _dlclose(handle: *mut c_void) -> c_int;
    #[link_name = "dlsym"]
    fn _dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    #[link_name = "dlerror"]
    fn _dlerror() -> *mut c_char;
}

/// FileHandle serves the purpose of not having direct access to the raw pointer
/// handle returned by dlopen outside of this library.
/// Every FileHandle corresponds to exactly one dlopen/dlclose calling pair.
/// As such, there is no implementation of the Copy-Trait
pub struct FileHandle {
    _private: *mut c_void,
}

impl FileHandle {
    /// Creates a handle that does not represent a file opened with dlopen.
    /// Use case: replacing a FileHandle field in a struct.
    pub fn invalid() -> Self {
        FileHandle {
            _private: std::ptr::null_mut(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self._private.is_null()
    }
}

// flags that can be passed to dlopen
pub const RTLD_LAZY: i32 = 0x00001;
pub const RTLD_NOW: i32 = 0x00002;
pub const RTLD_BINDING_MASK: i32 = 0x00003;
pub const RTLD_NOLOAD: i32 = 0x00004;
pub const RTLD_DEEPBIND: i32 = 0x00008;
pub const RTLD_GLOBAL: i32 = 0x00100;
pub const RTLD_LOCAL: i32 = 0;
pub const RTLD_NODELETE: i32 = 0x01000;

pub fn dlopen(filename: &str, flags: i32) -> Result<FileHandle, Box<dyn Error>> {
    if filename == "" {
        let handle = unsafe {
            FileHandle {
                _private: _dlopen(std::ptr::null(), flags),
            }
        };
        return Ok(handle);
    } else {
        let cstr_filename;
        match CString::new(filename) {
            Err(e) => return Err(Box::new(e)),
            Ok(value) => {
                cstr_filename = value;
            }
        }

        let handle = unsafe {
            FileHandle {
                _private: _dlopen(cstr_filename.as_ptr(), flags),
            }
        };
        return Ok(handle);
    }
}

pub fn dlclose(handle: FileHandle) -> i32 {
    unsafe { _dlclose(handle._private) }
}

pub fn dlsym(handle: &FileHandle, symbol: &str) -> Result<*mut c_void, Box<dyn Error>> {
    let cstr_symbol;
    match CString::new(symbol) {
        Err(e) => {
            return Err(Box::new(e));
        }
        Ok(value) => {
            cstr_symbol = value;
        }
    }

    let symbol_handle = unsafe { _dlsym(handle._private, cstr_symbol.as_ptr()) };
    Ok(symbol_handle)
}

pub fn dlerror() -> Result<Option<String>, std::str::Utf8Error> {
    unsafe {
        let message = _dlerror();
        if message.is_null() {
            return Ok(None);
        }

        let message = std::ffi::CStr::from_ptr(message);
        match message.to_str() {
            Err(err) => Err(err),
            Ok(message) => {
                let message = message.to_owned();
                Ok(Some(message))
            }
        }
    }
}
