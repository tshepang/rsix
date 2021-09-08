//! Functions which operate on file descriptors.

use crate::imp;
use crate::io::{self, OwnedFd};
#[cfg(all(libc, not(any(target_os = "wasi", target_os = "fuchsia"))))]
use crate::std_ffi::OsString;
use io_lifetimes::AsFd;

#[cfg(not(target_os = "wasi"))]
pub use imp::io::DupFlags;

/// `ioctl(fd, FIONREAD)`—Returns the number of bytes ready to be read.
///
/// The result of this function gets silently coerced into a C `int`
/// by the OS, so it may contain a wrapped value.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/ioctl_tty.2.html
#[cfg(not(target_os = "redox"))]
#[inline]
pub fn ioctl_fionread<Fd: AsFd>(fd: &Fd) -> io::Result<u64> {
    let fd = fd.as_fd();
    imp::syscalls::ioctl_fionread(fd)
}

/// `isatty(fd)`—Tests whether a file descriptor refers to a terminal.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/isatty.html
/// [Linux]: https://man7.org/linux/man-pages/man3/isatty.3.html
#[inline]
pub fn isatty<Fd: AsFd>(fd: &Fd) -> bool {
    let fd = fd.as_fd();
    imp::syscalls::isatty(fd)
}

/// Returns a pair of booleans indicating whether the file descriptor is
/// readable and/or writeable, respectively.
///
/// Unlike [`is_file_read_write`], this correctly detects whether sockets
/// have been shutdown, partially or completely.
///
/// [`is_file_read_write`]: crate::fs::is_file_read_write
#[cfg(not(target_os = "redox"))]
#[inline]
pub fn is_read_write<Fd: AsFd>(fd: &Fd) -> io::Result<(bool, bool)> {
    let fd = fd.as_fd();
    imp::syscalls::is_read_write(fd)
}

/// `dup(fd)`—Creates a new `OwnedFd` instance that shares the same
/// underlying [file description] as `fd`.
///
/// Note that this function does not set the `O_CLOEXEC` flag. To do a `dup`
/// that does set `O_CLOEXEC`, use [`fcntl_dupfd_cloexec`].
///
/// POSIX guarantees that `dup` will use the lowest unused file descriptor,
/// however it is not safe in general to rely on this, as file descriptors may
/// be unexpectedly allocated on other threads or in libraries.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [file description]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_258
/// [`fcntl_dupfd_cloexec`]: crate::fs::fcntl_dupfd_cloexec
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/dup.html
/// [Linux]: https://man7.org/linux/man-pages/man2/dup.2.html
#[cfg(not(target_os = "wasi"))]
#[inline]
pub fn dup<Fd: AsFd>(fd: &Fd) -> io::Result<OwnedFd> {
    let fd = fd.as_fd();
    imp::syscalls::dup(fd)
}

/// `dup2(fd, new)`—Creates a new `OwnedFd` instance that shares the
/// same underlying [file description] as the existing `OwnedFd` instance,
/// closing `new` and reusing its file descriptor.
///
/// Note that this function does not set the `O_CLOEXEC` flag. To do a `dup2`
/// that does set `O_CLOEXEC`, use [`dup2_with`] with [`DupFlags::CLOEXEC`] on
/// platforms which support it.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [file description]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_258
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/dup2.html
/// [Linux]: https://man7.org/linux/man-pages/man2/dup2.2.html
#[cfg(not(target_os = "wasi"))]
#[inline]
pub fn dup2<Fd: AsFd>(fd: &Fd, new: &OwnedFd) -> io::Result<()> {
    let fd = fd.as_fd();
    imp::syscalls::dup2(fd, new)
}

/// `dup3(fd, new, flags)`—Creates a new `OwnedFd` instance that shares the
/// same underlying [file description] as the existing `OwnedFd` instance,
/// closing `new` and reusing its file descriptor, with flags.
///
/// `dup2_with` is the same as `dup2` but adds an additional flags operand.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [file description]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_258
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/dup2.html
/// [Linux]: https://man7.org/linux/man-pages/man2/dup2.2.html
#[cfg(not(target_os = "wasi"))]
#[inline]
#[doc(alias = "dup3")]
pub fn dup2_with<Fd: AsFd>(fd: &Fd, new: &OwnedFd, flags: DupFlags) -> io::Result<()> {
    let fd = fd.as_fd();
    imp::syscalls::dup2_with(fd, new, flags)
}

/// `ttyname_r(fd)`
///
/// If `reuse` is non-empty, reuse its buffer to store the result if possible.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/ttyname.html
/// [Linux]: https://man7.org/linux/man-pages/man3/ttyname.3.html
// TODO: Implement ttyname for linux_raw
#[cfg(all(libc, not(any(target_os = "fuchsia", target_os = "wasi"))))]
#[inline]
pub fn ttyname<Fd: AsFd>(dirfd: &Fd, reuse: OsString) -> io::Result<OsString> {
    let dirfd = dirfd.as_fd();
    imp::syscalls::ttyname(dirfd, reuse)
}
