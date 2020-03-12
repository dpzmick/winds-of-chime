use std::os::raw::c_char;

#[macro_export]
macro_rules! static_str_ref {
    ($s:expr) => (
        unsafe { crate::ffi::StrRef::from_static_ptr(crate::static_cstr!($s)) }
    );
}

/// std:ffi::CStr is not repr transparent
#[repr(transparent)]
pub struct StrRef<'a>(*const c_char, PhantomData<&'a [c_char]>);

impl StrRef<'static> {
    pub unsafe fn from_static_ptr(ptr: *const c_char) -> Self {
        Self(ptr, PhantomData)
    }

    pub unsafe fn from_static_cstr(cstr: &'static CStr) -> Self {
        Self(cstr.as_ptr(), PhantomData)
    }
}

impl<'a> StrRef<'a> {
    pub fn from_ptr(ptr: *const c_char) -> Self {
        Self(ptr, PhantomData)
    }

    pub fn from_cstr(cstr: &'a CStr) -> Self {
        Self(cstr.as_ptr(), PhantomData)
    }

    pub fn to_ptr(self) -> *const c_char {
        self.0
    }
}

// it is safe to copy pointers
impl<'a> Copy for StrRef<'a> { }

impl<'a> Clone for StrRef<'a> {
    fn clone(&self) -> Self {
        *self
    }
}

