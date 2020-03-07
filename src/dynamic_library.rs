use std::os::raw::*;

extern "C" {
    fn dlopen(fname: *const c_char, flag: c_int) -> *mut c_void;
    fn dlerror() -> *const c_char;
    fn dlsym(lib: *mut c_void, symbol: *const c_char) -> *const c_void;
    fn dlclose(lib: *mut c_void);
}

pub struct DynamicLibrary {
    handle: *mut c_void,
}

impl DynamicLibrary {
    pub fn open(filename: &std::ffi::CStr) -> Result<Self, String> {
        let now = 0x00002;
        let local = 0;
        let deepbind = 0x00008;
        let flag = now | local | deepbind;

        unsafe { dlerror() };
        let handle = unsafe { dlopen(filename.as_ptr(), flag) };
        if handle.is_null() { return Err( Self::errstr() ); }

        Ok(Self {
            handle
        })
    }

    pub unsafe fn sym(&self, name: &std::ffi::CStr) -> Result<*const c_void, String> {
        let sym = dlsym(self.handle, name.as_ptr());
        if sym.is_null() { return Err( Self::errstr() ) }
        return Ok( sym )
    }

    fn errstr() -> String {
        unsafe {
            let s = dlerror();
            let s = std::ffi::CStr::from_ptr(s);
            return s.to_str().unwrap().to_string();
        }
    }
}

impl Drop for DynamicLibrary {
    fn drop(&mut self) {
        unsafe {
            dlclose(self.handle)
        }
    }
}
