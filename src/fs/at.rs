//! POSIX-style `*at` functions.

#[cfg(any(target_os = "ios", target_os = "macos"))]
use crate::fs::CloneFlags;
#[cfg(any(linux_raw, all(libc, any(target_os = "android", target_os = "linux"))))]
use crate::fs::RenameFlags;
use crate::io::{self, OwnedFd};
use crate::std_ffi::{CStr, OsString};
use crate::std_os_ffi::OsStringExt;
use crate::{imp, path};
#[cfg(not(any(
    target_os = "ios",
    target_os = "macos",
    target_os = "redox",
    target_os = "wasi",
)))]
use imp::fs::Dev;
use imp::fs::{Access, AtFlags, Mode, OFlags, Stat};
use imp::time::Timespec;
use io_lifetimes::{AsFd, BorrowedFd};

/// `openat(dirfd, path, oflags, mode)`—Opens a file.
///
/// POSIX guarantees that `openat` will use the lowest unused file descriptor,
/// however it is not safe in general to rely on this, as file descriptors may
/// be unexpectedly allocated on other threads or in libraries.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/openat.html
/// [Linux]: https://man7.org/linux/man-pages/man2/open.2.html
#[inline]
pub fn openat<P: path::Arg, Fd: AsFd>(
    dirfd: &Fd,
    path: P,
    oflags: OFlags,
    create_mode: Mode,
) -> io::Result<OwnedFd> {
    let dirfd = dirfd.as_fd();
    path.into_with_c_str(|path| imp::syscalls::openat(dirfd, path, oflags, create_mode))
}

/// `readlinkat(fd, path)`—Reads the contents of a symlink.
///
/// If `reuse` is non-empty, reuse its buffer to store the result if possible.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/readlinkat.html
/// [Linux]: https://man7.org/linux/man-pages/man2/readlinkat.2.html
#[inline]
pub fn readlinkat<P: path::Arg, Fd: AsFd>(
    dirfd: &Fd,
    path: P,
    reuse: OsString,
) -> io::Result<OsString> {
    let dirfd = dirfd.as_fd();
    path.into_with_c_str(|path| _readlinkat(dirfd, path, reuse))
}

fn _readlinkat(dirfd: BorrowedFd<'_>, path: &CStr, reuse: OsString) -> io::Result<OsString> {
    // This code would benefit from having a better way to read into
    // uninitialized memory, but that requires `unsafe`.
    let mut buffer = reuse.into_vec();
    buffer.clear();
    buffer.resize(256, 0_u8);

    loop {
        let nread = imp::syscalls::readlinkat(dirfd, path, &mut buffer)?;

        let nread = nread as usize;
        assert!(nread <= buffer.len());
        if nread < buffer.len() {
            buffer.resize(nread, 0_u8);
            return Ok(OsString::from_vec(buffer));
        }
        buffer.resize(buffer.len() * 2, 0_u8);
    }
}

/// `mkdirat(fd, path, mode)`—Creates a directory.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/mkdirat.html
/// [Linux]: https://man7.org/linux/man-pages/man2/mkdirat.2.html
#[inline]
pub fn mkdirat<P: path::Arg, Fd: AsFd>(dirfd: &Fd, path: P, mode: Mode) -> io::Result<()> {
    let dirfd = dirfd.as_fd();
    path.into_with_c_str(|path| imp::syscalls::mkdirat(dirfd, path, mode))
}

/// `linkat(old_dirfd, old_path, new_dirfd, new_path, flags)`—Creates a hard
/// link.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/linkat.html
/// [Linux]: https://man7.org/linux/man-pages/man2/linkat.2.html
#[inline]
pub fn linkat<P: path::Arg, Q: path::Arg, PFd: AsFd, QFd: AsFd>(
    old_dirfd: &PFd,
    old_path: P,
    new_dirfd: &QFd,
    new_path: Q,
    flags: AtFlags,
) -> io::Result<()> {
    let old_dirfd = old_dirfd.as_fd();
    let new_dirfd = new_dirfd.as_fd();
    old_path.into_with_c_str(|old_path| {
        new_path.into_with_c_str(|new_path| {
            imp::syscalls::linkat(old_dirfd, old_path, new_dirfd, new_path, flags)
        })
    })
}

/// `unlinkat(fd, path, flags)`—Unlinks a file or remove a directory.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/unlinkat.html
/// [Linux]: https://man7.org/linux/man-pages/man2/unlinkat.2.html
#[inline]
pub fn unlinkat<P: path::Arg, Fd: AsFd>(dirfd: &Fd, path: P, flags: AtFlags) -> io::Result<()> {
    let dirfd = dirfd.as_fd();
    path.into_with_c_str(|path| imp::syscalls::unlinkat(dirfd, path, flags))
}

/// `renameat(old_dirfd, old_path, new_dirfd, new_path)`—Renames a file or
/// directory.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/renameat.html
/// [Linux]: https://man7.org/linux/man-pages/man2/renameat.2.html
#[inline]
pub fn renameat<P: path::Arg, Q: path::Arg, PFd: AsFd, QFd: AsFd>(
    old_dirfd: &PFd,
    old_path: P,
    new_dirfd: &QFd,
    new_path: Q,
) -> io::Result<()> {
    let old_dirfd = old_dirfd.as_fd();
    let new_dirfd = new_dirfd.as_fd();
    old_path.into_with_c_str(|old_path| {
        new_path.into_with_c_str(|new_path| {
            imp::syscalls::renameat(old_dirfd, old_path, new_dirfd, new_path)
        })
    })
}

/// `renameat2(old_dirfd, old_path, new_dirfd, new_path, flags)`—Renames a
/// file or directory.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/renameat2.2.html
#[cfg(any(linux_raw, all(libc, any(target_os = "android", target_os = "linux"))))]
#[inline]
#[doc(alias = "renameat2")]
pub fn renameat_with<P: path::Arg, Q: path::Arg, PFd: AsFd, QFd: AsFd>(
    old_dirfd: &PFd,
    old_path: P,
    new_dirfd: &QFd,
    new_path: Q,
    flags: RenameFlags,
) -> io::Result<()> {
    let old_dirfd = old_dirfd.as_fd();
    let new_dirfd = new_dirfd.as_fd();
    old_path.into_with_c_str(|old_path| {
        new_path.into_with_c_str(|new_path| {
            imp::syscalls::renameat2(old_dirfd, old_path, new_dirfd, new_path, flags)
        })
    })
}

/// `symlinkat(old_dirfd, old_path, new_dirfd, new_path)`—Creates a symlink.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/symlinkat.html
/// [Linux]: https://man7.org/linux/man-pages/man2/symlinkat.2.html
#[inline]
pub fn symlinkat<P: path::Arg, Q: path::Arg, Fd: AsFd>(
    old_path: P,
    new_dirfd: &Fd,
    new_path: Q,
) -> io::Result<()> {
    let new_dirfd = new_dirfd.as_fd();
    old_path.into_with_c_str(|old_path| {
        new_path.into_with_c_str(|new_path| imp::syscalls::symlinkat(old_path, new_dirfd, new_path))
    })
}

/// `fstatat(dirfd, path, flags)`—Queries metadata for a file or directory.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/fstatat.html
/// [Linux]: https://man7.org/linux/man-pages/man2/fstatat.2.html
#[inline]
#[doc(alias = "fstatat")]
pub fn statat<P: path::Arg, Fd: AsFd>(dirfd: &Fd, path: P, flags: AtFlags) -> io::Result<Stat> {
    let dirfd = dirfd.as_fd();
    path.into_with_c_str(|path| imp::syscalls::statat(dirfd, path, flags))
}

/// `faccessat(dirfd, path, access, flags)`—Tests permissions for a file or
/// directory.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/faccessat.html
/// [Linux]: https://man7.org/linux/man-pages/man2/faccessat.2.html
#[inline]
#[doc(alias = "faccessat")]
pub fn accessat<P: path::Arg, Fd: AsFd>(
    dirfd: &Fd,
    path: P,
    access: Access,
    flags: AtFlags,
) -> io::Result<()> {
    let dirfd = dirfd.as_fd();
    path.into_with_c_str(|path| imp::syscalls::accessat(dirfd, path, access, flags))
}

/// `utimensat(dirfd, path, times, flags)`—Sets file or directory timestamps.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/utimensat.html
/// [Linux]: https://man7.org/linux/man-pages/man2/utimensat.2.html
#[inline]
pub fn utimensat<P: path::Arg, Fd: AsFd>(
    dirfd: &Fd,
    path: P,
    times: &[Timespec; 2],
    flags: AtFlags,
) -> io::Result<()> {
    let dirfd = dirfd.as_fd();
    path.into_with_c_str(|path| imp::syscalls::utimensat(dirfd, path, times, flags))
}

/// `fchmodat(dirfd, path, mode, 0)`—Sets file or directory permissions.
///
/// The flags argument is fixed to 0, so `AT_SYMLINK_NOFOLLOW` is not
/// supported. <details>Platform support for this flag varies widely.</details>
///
/// Note that this implementation does not support `O_PATH` file descriptors,
/// even on platforms where the host libc emulates it.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/fchmodat.html
/// [Linux]: https://man7.org/linux/man-pages/man2/fchmodat.2.html
#[cfg(not(target_os = "wasi"))]
#[inline]
#[doc(alias = "fchmodat")]
pub fn chmodat<P: path::Arg, Fd: AsFd>(dirfd: &Fd, path: P, mode: Mode) -> io::Result<()> {
    let dirfd = dirfd.as_fd();
    path.into_with_c_str(|path| imp::syscalls::chmodat(dirfd, path, mode))
}

/// `fclonefileat(src, dst_dir, dst, flags)`—Efficiently copies between files.
///
/// # References
///  - [Apple]
///
/// [Apple]: https://opensource.apple.com/source/xnu/xnu-3789.21.4/bsd/man/man2/clonefile.2.auto.html
#[cfg(any(target_os = "ios", target_os = "macos"))]
#[inline]
pub fn fclonefileat<Fd: AsFd, DstFd: AsFd, P: path::Arg>(
    src: &Fd,
    dst_dir: &DstFd,
    dst: P,
    flags: CloneFlags,
) -> io::Result<()> {
    let srcfd = src.as_fd();
    let dst_dirfd = dst_dir.as_fd();
    dst.into_with_c_str(|dst| {
        imp::syscalls::fclonefileat(srcfd.as_fd(), dst_dirfd.as_fd(), &dst, flags)
    })
}

/// `mknodat(dirfd, path, mode, dev)`—Creates special or normal files.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/mknodat.html
/// [Linux]: https://man7.org/linux/man-pages/man2/mknodat.2.html
#[cfg(not(any(
    target_os = "ios",
    target_os = "macos",
    target_os = "redox",
    target_os = "wasi",
)))]
#[inline]
pub fn mknodat<P: path::Arg, Fd: AsFd>(
    dirfd: &Fd,
    path: P,
    mode: Mode,
    dev: Dev,
) -> io::Result<()> {
    let dirfd = dirfd.as_fd();
    path.into_with_c_str(|path| imp::syscalls::mknodat(dirfd, path, mode, dev))
}
