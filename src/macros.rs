macro_rules! bytes_nul_terminate {
    ($s:expr) => ( concat!($s, "\0").as_bytes() );
}

// macro_rules! cstr_lit {
//     ($s:expr) => ( concat!($s, "\0").as_bytes() as *const std::os::raw::c_char; );
// }

macro_rules! ffi_cstr { // wow this sucks
    ($s:expr) => (
        std::ffi::CStr::from_bytes_with_nul(bytes_nul_terminate!($s)).expect("Failed to format ffi cstr")
    );
}
