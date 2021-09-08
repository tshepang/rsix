//! The `madvise` function.
//!
//! # Safety
//!
//! `madvise` operates on a raw pointer. Some forms of `madvise` may
//! mutate the memory or have other side effects.
#![allow(unsafe_code)]

use crate::{imp, io};
use core::ffi::c_void;

pub use imp::io::Advice;

/// `posix_madvise(addr, len, advice)`—Declares an expected access pattern
/// for a memory-mapped file.
///
/// # Safety
///
/// `addr` must be a valid pointer to memory that is appropriate to
/// call `posix_madvise` on. Some forms of `advice` may mutate the memory
/// or evoke a variety of side-effects on the mapping and/or the file.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/posix_madvise.html
/// [Linux]: https://man7.org/linux/man-pages/man2/posix_madvise.2.html
#[inline]
#[doc(alias = "posix_madvise")]
pub unsafe fn madvise(addr: *mut c_void, len: usize, advice: Advice) -> io::Result<()> {
    imp::syscalls::madvise(addr, len, advice)
}
