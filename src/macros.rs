#[macro_export]
macro_rules! bytes_nul_terminate {
    ($s:expr) => ( concat!($s, "\0").as_bytes() );
}

#[macro_export]
macro_rules! static_cstr {
    ($s:expr) => ( bytes_nul_terminate!($s).as_ptr() as *const std::os::raw::c_char );
}

#[macro_export]
macro_rules! static_ffi_cstr {
    ($s:expr) => (
        unsafe { std::ffi::CStr::from_ptr(static_cstr!($s)) }
    );
}
