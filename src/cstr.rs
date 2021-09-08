/// A simple macro for `CStr` literals that doesn't depend on `proc_macro2` or
/// `syn` or anything else.
#[cfg(debug_assertions)]
#[allow(unused_macros)]
macro_rules! cstr {
    ($str:literal) => {
        crate::std_ffi::CStr::from_bytes_with_nul(concat!($str, "\0").as_bytes()).unwrap()
    };
}
#[cfg(not(debug_assertions))]
#[allow(unused_macros)]
macro_rules! cstr {
    ($str:literal) => {{
        #[allow(unsafe_code, unused_unsafe)]
        unsafe {
            crate::std_ffi::CStr::from_bytes_with_nul_unchecked(concat!($str, "\0").as_bytes())
        }
    }};
}

#[test]
fn test_cstr() {
    use crate::std_ffi::CString;
    assert_eq!(cstr!(""), CString::new("").unwrap().as_c_str());
    assert_eq!(cstr!("").to_owned(), CString::new("").unwrap());
    assert_eq!(cstr!("hello"), CString::new("hello").unwrap().as_c_str());
    assert_eq!(cstr!("hello").to_owned(), CString::new("hello").unwrap());
}
