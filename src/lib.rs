//! `rsix` provides efficient memory-safe and [I/O-safe] wrappers to
//! POSIX-like, Unix-like, and Linux syscalls.
//!
//! The wrappers perform the following tasks:
//!  - Error values are translated to [`Result`]s.
//!  - Buffers are passed as Rust slices.
//!  - Out-parameters are presented as return values.
//!  - Path arguments use [`Arg`], so they accept any string type.
//!  - File descriptors are passed and returned via [`AsFd`] and [`OwnedFd`]
//!    instead of bare integers, ensuring I/O safety.
//!  - Constants use `enum`s and [`bitflags`] types.
//!  - Multiplexed functions (eg. `fcntl`, `ioctl`, etc.) are de-multiplexed.
//!  - Variadic functions (eg. `openat`, etc.) are presented as non-variadic.
//!  - Functions and types which need `l` prefixes or `64` suffixes to enable
//!    large-file support are used automatically, and file sizes and offsets
//!    are presented as `i64` and `u64`.
//!  - Behaviors that depend on the sizes of C types like `long` are hidden.
//!  - In some places, more human-friendly and less historical-accident names
//!    are used.
//!
//! Things they don't do include:
//!  - Detecting whether functions are supported at runtime.
//!  - Hiding significant differences between platforms.
//!  - Restricting ambient authorities.
//!  - Imposing sandboxing features such as filesystem path or network address
//!    sandboxing.
//!
//! See [`cap-std`], [`system-interface`], and [`io-streams`] for libraries
//! which do hide significant differences between platforms, and [`cap-std`]
//! which does perform sandboxing and restricts ambient authorities.
//!
//! [`cap-std`]: https://crates.io/crates/cap-std
//! [`system-interface`]: https://crates.io/crates/system-interface
//! [`io-streams`]: https://crates.io/crates/io-streams
//! [`std`]: https://doc.rust-lang.org/std/
//! [`getrandom`]: https://crates.io/crates/getrandom
//! [`bitflags`]: https://crates.io/crates/bitflags
//! [`AsFd`]: https://docs.rs/io-lifetimes/latest/io_lifetimes/trait.AsFd.html
//! [`OwnedFd`]: https://docs.rs/io-lifetimes/latest/io_lifetimes/struct.OwnedFd.html
//! [io-lifetimes crate]: https://crates.io/crates/io-lifetimes
//! [I/O-safe]: https://github.com/rust-lang/rfcs/pull/3128
//! [`Result`]: https://docs.rs/rsix/latest/rsix/io/type.Result.html
//! [`Arg`]: https://docs.rs/rsix/latest/rsix/path/trait.Arg.html

#![deny(missing_docs)]
#![cfg_attr(linux_raw, deny(unsafe_code))]
#![cfg_attr(linux_raw_inline_asm, feature(asm))]
#![cfg_attr(rustc_attrs, feature(rustc_attrs))]
#![cfg_attr(target_os = "wasi", feature(wasi_ext))]
#![cfg_attr(
    all(linux_raw_inline_asm, target_arch = "x86"),
    feature(naked_functions)
)]
#![cfg_attr(io_lifetimes_use_std, feature(io_safety))]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "no_std_demo", feature(core_intrinsics))]

extern crate alloc;

/// Re-export `io_lifetimes` since we use its types in our public API, so
/// that our users don't need to do anything special to use the same version.
pub use io_lifetimes;

#[macro_use]
pub(crate) mod cstr;
#[macro_use]
pub(crate) mod const_assert;

mod imp;

pub mod fs;
pub mod io;
#[cfg(not(any(target_os = "redox", target_os = "wasi")))] // WASI doesn't support `net` yet.
pub mod net;
pub mod path;
pub mod process;
pub mod rand;
pub mod thread;
pub mod time;

/// Convert a `&T` into a `*const T` without using an `as`.
#[inline]
#[allow(dead_code)]
const fn as_ptr<T>(t: &T) -> *const T {
    t
}

/// Convert a `&mut T` into a `*mut T` without using an `as`.
#[inline]
#[allow(dead_code)]
fn as_mut_ptr<T>(t: &mut T) -> *mut T {
    t
}

#[cfg(feature = "no_std_demo")]
use cty as c_types;
#[cfg(feature = "std")]
use std::os::raw as c_types;

#[cfg(feature = "no_std_demo")]
use embedded_ffi as std_ffi;
#[cfg(feature = "std")]
use std::ffi as std_ffi;

#[cfg(feature = "no_std_demo")]
use embedded_ffi as std_os_ffi;
#[cfg(all(target_os = "hermit", feature = "std"))]
use std::os::hermit::ext::ffi as std_os_ffi;
#[cfg(all(unix, feature = "std"))]
use std::os::unix::ffi as std_os_ffi;
#[cfg(all(target_os = "wasi", feature = "std"))]
use std::os::wasi::ffi as std_os_ffi;

#[cfg(all(unix, feature = "std"))]
use std::os::unix::fs as std_os_fs;
#[cfg(all(target_os = "wasi", feature = "std"))]
use std::os::wasi::fs as std_os_fs;

#[cfg(feature = "no_std_demo")]
use io_lifetimes::std_os_io;
#[cfg(all(unix, feature = "std"))]
use std::os::unix::io as std_os_io;
#[cfg(all(target_os = "wasi", feature = "std"))]
use std::os::wasi::io as std_os_io;

#[cfg(feature = "std")]
use std::path as std_path;
#[cfg(feature = "no_std_demo")]
use unix_path as std_path;

#[cfg(feature = "no_std_demo")]
trait AsOsStr {
    fn as_os_str(&self) -> &crate::std_ffi::OsStr;
}
#[cfg(feature = "no_std_demo")]
trait IntoOsString {
    fn into_os_string(self) -> crate::std_ffi::OsString;
}
#[cfg(feature = "no_std_demo")]
impl AsOsStr for unix_path::Path {
    #[inline]
    fn as_os_str(&self) -> &crate::std_ffi::OsStr {
        crate::std_os_ffi::OsStrExt::from_bytes(self.as_unix_str().as_bytes())
    }
}
#[cfg(feature = "no_std_demo")]
impl AsOsStr for unix_path::Component<'_> {
    #[inline]
    fn as_os_str(&self) -> &crate::std_ffi::OsStr {
        crate::std_os_ffi::OsStrExt::from_bytes(self.as_unix_str().as_bytes())
    }
}
#[cfg(feature = "no_std_demo")]
impl IntoOsString for unix_path::PathBuf {
    #[inline]
    fn into_os_string(self) -> crate::std_ffi::OsString {
        crate::std_os_ffi::OsStringExt::from_vec(self.into_unix_string().into_vec())
    }
}

#[cfg(feature = "no_std_demo")]
use core2::io as std_io;
#[cfg(feature = "std")]
use std::io as std_io;

#[cfg(feature = "no_std_demo")]
use no_std_net as std_net;
#[cfg(feature = "std")]
use std::net as std_net;
