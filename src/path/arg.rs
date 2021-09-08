use crate::io;
use crate::path::DecInt;
use crate::std_ffi::{CStr, CString, OsStr, OsString};
use crate::std_os_ffi::{OsStrExt, OsStringExt};
use crate::std_path::{Component, Components, Iter, Path, PathBuf};
#[cfg(feature = "no_std_demo")]
use crate::{AsOsStr, IntoOsString};
use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::str;

/// A trait for passing path arguments.
///
/// This is similar to [`AsRef`]`<`[`Path`]`>`, but is implemented for more
/// kinds of strings and can convert into more kinds of strings.
///
/// # Example
///
/// ```rust
/// use rsix::io;
/// use rsix::path::Arg;
/// use std::ffi::CStr;
///
/// pub fn touch<P: Arg>(path: P) -> io::Result<()> {
///     let path = path.into_c_str()?;
///     _touch(&path)
/// }
///
/// fn _touch(path: &CStr) -> io::Result<()> {
///     // implementation goes here
///     Ok(())
/// }
/// ```
///
/// Users can then call `touch("foo")`, `touch(cstr!("foo"))`,
/// `touch(Path::new("foo"))`, or many other things.
///
/// [`AsRef`]: std::convert::AsRef
pub trait Arg {
    /// Returns a view of this string as a string slice.
    fn as_str(&self) -> io::Result<&str>;

    /// Returns a potentially-lossy rendering of this string as a `Cow<str>`.
    #[cfg(feature = "std")]
    fn to_string_lossy(&self) -> Cow<str>;

    /// Returns a view of this string as a maybe-owned [`CStr`].
    #[cfg(not(windows))]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>>;

    /// Consumes `self` and returns a view of this string as a maybe-owned
    /// [`CStr`].
    #[cfg(not(windows))]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b;

    /// Returns a view of this string as a byte slice.
    #[cfg(not(windows))]
    fn as_maybe_utf8_bytes(&self) -> &[u8];

    /// Returns a view of this string as a maybe-owned [`OsStr`].
    #[cfg(windows)]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>>;

    /// Runs a closure with `self` passed in as a `&CStr`.
    #[cfg(not(windows))]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>;
}

impl Arg for &str {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        Ok(self)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        Cow::Borrowed(self)
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Owned(
            CString::new(self.as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            CString::new(self).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        with_c_str(self.as_bytes(), f)
    }
}

#[cfg(feature = "std")]
impl Arg for &String {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        Ok(self)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        Cow::Borrowed(self)
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Owned(
            CString::new(String::as_str(self).as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        self.as_str().into_c_str()
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        with_c_str(self.as_bytes(), f)
    }
}

#[cfg(feature = "std")]
impl Arg for String {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        Ok(self)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        Cow::Borrowed(self)
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Owned(
            CString::new(self.as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            CString::new(self).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        f(&CString::new(self).map_err(|_cstr_err| io::Error::INVAL)?)
    }
}

impl Arg for &OsStr {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.to_str().ok_or(io::Error::INVAL)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        OsStr::to_string_lossy(self)
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Owned(
            CString::new(self.as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            CString::new(self.as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        with_c_str(self.as_bytes(), f)
    }
}

impl Arg for &OsString {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        OsString::as_os_str(self).to_str().ok_or(io::Error::INVAL)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        self.as_os_str().to_string_lossy()
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Owned(
            CString::new(OsString::as_os_str(self).as_bytes())
                .map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        self.as_os_str().into_c_str()
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        with_c_str(self.as_bytes(), f)
    }
}

impl Arg for OsString {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.as_os_str().to_str().ok_or(io::Error::INVAL)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        self.as_os_str().to_string_lossy()
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Owned(
            CString::new(self.as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            CString::new(self.into_vec()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        f(&CString::new(self.into_vec()).map_err(|_cstr_err| io::Error::INVAL)?)
    }
}

impl Arg for &Path {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.as_os_str().to_str().ok_or(io::Error::INVAL)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        Path::to_string_lossy(self)
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Owned(
            CString::new(self.as_os_str().as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            CString::new(self.as_os_str().as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self.as_os_str().as_bytes()
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        with_c_str(self.as_os_str().as_bytes(), f)
    }
}

impl Arg for &PathBuf {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        PathBuf::as_path(self)
            .as_os_str()
            .to_str()
            .ok_or(io::Error::INVAL)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        self.as_path().to_string_lossy()
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Owned(
            CString::new(PathBuf::as_path(self).as_os_str().as_bytes())
                .map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        self.as_path().into_c_str()
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        PathBuf::as_path(self).as_os_str().as_bytes()
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        with_c_str(self.as_os_str().as_bytes(), f)
    }
}

impl Arg for PathBuf {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.as_os_str().to_str().ok_or(io::Error::INVAL)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        self.as_os_str().to_string_lossy()
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Owned(
            CString::new(self.as_os_str().as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            CString::new(self.into_os_string().into_vec()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self.as_os_str().as_bytes()
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        f(
            &CString::new(self.into_os_string().into_vec())
                .map_err(|_cstr_err| io::Error::INVAL)?,
        )
    }
}

impl Arg for &CStr {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.to_str().map_err(|_utf8_err| io::Error::INVAL)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        CStr::to_string_lossy(self)
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Borrowed(self))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Borrowed(self))
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self.to_bytes()
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        f(self)
    }
}

impl Arg for &CString {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        CString::as_c_str(self)
            .to_str()
            .map_err(|_utf8_err| io::Error::INVAL)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        CString::as_c_str(self).to_string_lossy()
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Borrowed(CString::as_c_str(self)))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Borrowed(self))
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self.to_bytes()
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        f(self)
    }
}

impl Arg for CString {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.to_str().map_err(|_utf8_err| io::Error::INVAL)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        self.as_c_str().to_string_lossy()
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Borrowed(self))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(self))
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        f(&self)
    }
}

impl<'a> Arg for Cow<'a, str> {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        Ok(self)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        Cow::Borrowed(self)
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Owned(
            CString::new(self.as_ref()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            match self {
                Cow::Owned(s) => CString::new(s),
                Cow::Borrowed(s) => CString::new(s),
            }
            .map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        with_c_str(self.as_bytes(), f)
    }
}

impl<'a> Arg for Cow<'a, OsStr> {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        (**self).to_str().ok_or(io::Error::INVAL)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        (**self).to_string_lossy()
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Owned(
            CString::new(self.as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            match self {
                Cow::Owned(os) => CString::new(os.into_vec()),
                Cow::Borrowed(os) => CString::new(os.as_bytes()),
            }
            .map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        with_c_str(self.as_bytes(), f)
    }
}

impl<'a> Arg for Cow<'a, CStr> {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.to_str().map_err(|_utf8_err| io::Error::INVAL)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        let borrow: &CStr = core::borrow::Borrow::borrow(self);
        borrow.to_string_lossy()
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Borrowed(self))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(self)
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self.to_bytes()
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        f(&self)
    }
}

impl<'a> Arg for Component<'a> {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.as_os_str().to_str().ok_or(io::Error::INVAL)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        self.as_os_str().to_string_lossy()
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Owned(
            CString::new(self.as_os_str().as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            CString::new(self.as_os_str().as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self.as_os_str().as_bytes()
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        with_c_str(self.as_os_str().as_bytes(), f)
    }
}

impl<'a> Arg for Components<'a> {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.as_path().to_str().ok_or(io::Error::INVAL)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        self.as_path().to_string_lossy()
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Owned(
            CString::new(self.as_path().as_os_str().as_bytes())
                .map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            CString::new(self.as_path().as_os_str().as_bytes())
                .map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self.as_path().as_os_str().as_bytes()
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        with_c_str(self.as_path().as_os_str().as_bytes(), f)
    }
}

impl<'a> Arg for Iter<'a> {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.as_path().to_str().ok_or(io::Error::INVAL)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        self.as_path().to_string_lossy()
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Owned(
            CString::new(self.as_path().as_os_str().as_bytes())
                .map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            CString::new(self.as_path().as_os_str().as_bytes())
                .map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self.as_path().as_os_str().as_bytes()
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        with_c_str(self.as_path().as_os_str().as_bytes(), f)
    }
}

impl Arg for &[u8] {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        str::from_utf8(self).map_err(|_utf8_err| io::Error::INVAL)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        String::from_utf8_lossy(self)
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Owned(
            CString::new(*self).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            CString::new(self).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        with_c_str(self, f)
    }
}

impl Arg for &Vec<u8> {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        str::from_utf8(self).map_err(|_utf8_err| io::Error::INVAL)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        String::from_utf8_lossy(self)
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Owned(
            CString::new(self.as_slice()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            CString::new(self.as_slice()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        with_c_str(self, f)
    }
}

impl Arg for Vec<u8> {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        str::from_utf8(self).map_err(|_utf8_err| io::Error::INVAL)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        String::from_utf8_lossy(self)
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Owned(
            CString::new(self.as_slice()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            CString::new(self).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        f(&CString::new(self).map_err(|_cstr_err| io::Error::INVAL)?)
    }
}

impl Arg for DecInt {
    #[inline]
    fn as_str(&self) -> io::Result<&str> {
        self.as_os_str().to_str().ok_or(io::Error::INVAL)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn to_string_lossy(&self) -> Cow<str> {
        Path::to_string_lossy(self)
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_cow_c_str(&self) -> io::Result<Cow<CStr>> {
        Ok(Cow::Owned(
            CString::new(self.as_os_str().as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_c_str<'b>(self) -> io::Result<Cow<'b, CStr>>
    where
        Self: 'b,
    {
        Ok(Cow::Owned(
            CString::new(self.as_os_str().as_bytes()).map_err(|_cstr_err| io::Error::INVAL)?,
        ))
    }

    #[cfg(not(windows))]
    #[inline]
    fn as_maybe_utf8_bytes(&self) -> &[u8] {
        self.as_os_str().as_bytes()
    }

    #[cfg(windows)]
    #[inline]
    fn as_os_str(&self) -> io::Result<Cow<OsStr>> {
        self.as_ref()
    }

    #[cfg(not(windows))]
    #[inline]
    fn into_with_c_str<T, F>(self, f: F) -> io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&CStr) -> io::Result<T>,
    {
        f(self.as_c_str())
    }
}

/// Runs a closure with `bytes` passed in as a `&CStr`.
#[inline]
fn with_c_str<T, F>(bytes: &[u8], f: F) -> io::Result<T>
where
    F: FnOnce(&CStr) -> io::Result<T>,
{
    // Most paths are less than this long. The rest can go through the dynamic
    // allocation path. If you're opening many files in a directory with a long
    // path, consider opening the directory and using openat to open the files
    // under it, which will avoid this, and is often faster in the OS as well.
    const SIZE: usize = 256;
    // Test with >= so that we have room for the trailing NUL.
    if bytes.len() >= SIZE {
        return with_c_str_slow_path(bytes, f);
    }
    let mut buffer: [u8; SIZE] = [0_u8; SIZE];
    // Copy the bytes in; the buffer already has zeros for the trailing NUL.
    buffer[..bytes.len()].copy_from_slice(bytes);
    f(CStr::from_bytes_with_nul(&buffer[..=bytes.len()]).map_err(|_cstr_err| io::Error::INVAL)?)
}

/// The slow path which handles any length. In theory OS's only support up
/// to `PATH_MAX`, but we let the OS enforce that.
#[cold]
fn with_c_str_slow_path<T, F>(bytes: &[u8], f: F) -> io::Result<T>
where
    F: FnOnce(&CStr) -> io::Result<T>,
{
    f(&CString::new(bytes).map_err(|_cstr_err| io::Error::INVAL)?)
}
