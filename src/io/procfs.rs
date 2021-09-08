//! Utilities for working with `/proc`, where Linux's `procfs` is typically
//! mounted. `/proc` serves as an adjunct to Linux's main syscall surface area,
//! providing additional features with an awkward interface.
//!
//! This module does a considerable amount of work to determine whether `/proc`
//! is mounted, with actual `procfs`, and without any additional mount points
//! on top of the paths we open.

use crate::fs::{
    cwd, fstat, fstatfs, major, openat, renameat, Mode, OFlags, Stat, PROC_SUPER_MAGIC,
};
use crate::io::{self, OwnedFd};
use crate::path::DecInt;
use crate::process::{getgid, getpid, getuid, Gid, RawGid, RawUid, Uid};
#[cfg(feature = "no_std_demo")]
use alloc::boxed::Box;
use io_lifetimes::{AsFd, BorrowedFd};
#[cfg(feature = "no_std_demo")]
use once_cell::race::OnceBox;
#[cfg(feature = "std")]
use once_cell::sync::OnceCell;

/// Linux's procfs always uses inode 1 for its root directory.
const PROC_ROOT_INO: u64 = 1;

// Identify an entry within "/proc", to determine which anomalies to
// check for.
enum Kind {
    Proc,
    Pid,
    Fd,
}

/// Check a subdirectory of "/proc" for anomalies.
fn check_proc_entry(
    kind: Kind,
    entry: BorrowedFd<'_>,
    proc_stat: Option<&Stat>,
    uid: RawUid,
    gid: RawGid,
) -> io::Result<Stat> {
    let entry_stat = fstat(&entry)?;
    check_proc_entry_with_stat(kind, entry, entry_stat, proc_stat, uid, gid)
}

/// Check a subdirectory of "/proc" for anomalies, using the provided `Stat`.
fn check_proc_entry_with_stat(
    kind: Kind,
    entry: BorrowedFd<'_>,
    entry_stat: Stat,
    proc_stat: Option<&Stat>,
    uid: RawUid,
    gid: RawGid,
) -> io::Result<Stat> {
    // Check the filesystem magic.
    check_procfs(entry)?;

    match kind {
        Kind::Proc => check_proc_root(entry, &entry_stat)?,
        Kind::Pid | Kind::Fd => check_proc_subdir(entry, &entry_stat, proc_stat)?,
    }

    // Check the ownership of the directory.
    if (entry_stat.st_uid, entry_stat.st_gid) != (uid, gid) {
        return Err(io::Error::NOTSUP);
    }

    // "/proc" directories are typically mounted r-xr-xr-x.
    // "/proc/self/fd" is r-x------. Allow them to have fewer permissions, but
    // not more.
    let expected_mode = if let Kind::Fd = kind { 0o500 } else { 0o555 };
    if entry_stat.st_mode & 0o777 & !expected_mode != 0 {
        return Err(io::Error::NOTSUP);
    }

    match kind {
        Kind::Fd => {
            // Check that the "/proc/self/fd" directory doesn't have any extraneous
            // links into it (which might include unexpected subdirectories).
            if entry_stat.st_nlink != 2 {
                return Err(io::Error::NOTSUP);
            }
        }
        Kind::Pid | Kind::Proc => {
            // Check that the "/proc" and "/proc/self" directories aren't empty.
            if entry_stat.st_nlink <= 2 {
                return Err(io::Error::NOTSUP);
            }
        }
    }

    Ok(entry_stat)
}

fn check_proc_root(entry: BorrowedFd<'_>, stat: &Stat) -> io::Result<()> {
    // We use `O_DIRECTORY` for proc directories, so open should fail if we
    // don't get a directory when we expect one.
    assert_eq!(stat.st_mode & Mode::IFMT.bits(), Mode::IFDIR.bits());

    // Check the root inode number.
    if stat.st_ino != PROC_ROOT_INO {
        return Err(io::Error::NOTSUP);
    }

    // Proc is a non-device filesystem, so check for major number 0.
    // <https://www.kernel.org/doc/Documentation/admin-guide/devices.txt>
    if major(stat.st_dev) != 0 {
        return Err(io::Error::NOTSUP);
    }

    // Check that "/proc" is a mountpoint.
    if !is_mountpoint(entry) {
        return Err(io::Error::NOTSUP);
    }

    Ok(())
}

fn check_proc_subdir(
    entry: BorrowedFd<'_>,
    stat: &Stat,
    proc_stat: Option<&Stat>,
) -> io::Result<()> {
    // We use `O_DIRECTORY` for proc directories, so open should fail if we
    // don't get a directory when we expect one.
    assert_eq!(stat.st_mode & Mode::IFMT.bits(), Mode::IFDIR.bits());

    check_proc_nonroot(stat, proc_stat)?;

    // Check that subdirectories of "/proc" are not mount points.
    if is_mountpoint(entry) {
        return Err(io::Error::NOTSUP);
    }

    Ok(())
}

fn check_proc_nonroot(stat: &Stat, proc_stat: Option<&Stat>) -> io::Result<()> {
    // Check that we haven't been linked back to the root of "/proc".
    if stat.st_ino == PROC_ROOT_INO {
        return Err(io::Error::NOTSUP);
    }

    // Check that we're still in procfs.
    if stat.st_dev != proc_stat.unwrap().st_dev {
        return Err(io::Error::NOTSUP);
    }

    Ok(())
}

/// Check that `file` is opened on a `procfs` filesystem.
fn check_procfs(file: BorrowedFd<'_>) -> io::Result<()> {
    let statfs = fstatfs(&file)?;
    let f_type = statfs.f_type;
    if f_type != PROC_SUPER_MAGIC {
        return Err(io::Error::NOTSUP);
    }

    Ok(())
}

/// Check whether the given directory handle is a mount point. We use a
/// `renameat` call that would otherwise fail, but which fails with `EXDEV`
/// first if it would cross a mount point.
fn is_mountpoint(file: BorrowedFd<'_>) -> bool {
    let err = renameat(&file, cstr!("../."), &file, cstr!(".")).unwrap_err();
    match err {
        io::Error::XDEV => true,  // the rename failed due to crossing a mount point
        io::Error::BUSY => false, // the rename failed normally
        _ => panic!("Unexpected error from `renameat`: {:?}", err),
    }
}

/// Returns a handle to Linux's `/proc` directory.
///
/// This ensures that `/proc` is procfs, that nothing is mounted on top of it,
/// and that it looks normal. It also returns the `Stat` of `/proc`.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man5/proc.5.html
fn proc() -> io::Result<(BorrowedFd<'static>, &'static Stat)> {
    static PROC: StaticFd = StaticFd::new();

    // `OnceBox` is "racey" in that the initialization function may run
    // multiple times. We're ok with that, since the initialization function
    // has no side effects.
    PROC.get_or_try_init(|| {
        let oflags = OFlags::NOFOLLOW
            | OFlags::PATH
            | OFlags::DIRECTORY
            | OFlags::CLOEXEC
            | OFlags::NOCTTY
            | OFlags::NOATIME;

        // Open "/proc".
        let proc = openat(&cwd(), cstr!("/proc"), oflags, Mode::empty())
            .map_err(|_err| io::Error::NOTSUP)?;
        let proc_stat = check_proc_entry(
            Kind::Proc,
            proc.as_fd(),
            None,
            Uid::ROOT.as_raw(),
            Gid::ROOT.as_raw(),
        )
        .map_err(|_err| io::Error::NOTSUP)?;

        Ok(new_static_fd(proc, proc_stat))
    })
    .map(|(fd, stat)| (fd.as_fd(), stat))
}

/// Returns a handle to Linux's `/proc/self` directory.
///
/// This ensures that `/proc/self` is procfs, that nothing is mounted on top of
/// it, and that it looks normal. It also returns the `Stat` of `/proc/self`.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man5/proc.5.html
fn proc_self() -> io::Result<(BorrowedFd<'static>, &'static Stat)> {
    static PROC_SELF: StaticFd = StaticFd::new();

    // The init function here may run multiple times; see above.
    PROC_SELF
        .get_or_try_init(|| {
            let (proc, proc_stat) = proc()?;

            let (uid, gid, pid) = (getuid(), getgid(), getpid());
            let oflags = OFlags::NOFOLLOW
                | OFlags::PATH
                | OFlags::DIRECTORY
                | OFlags::CLOEXEC
                | OFlags::NOCTTY
                | OFlags::NOATIME;

            // Open "/proc/self". Use our pid to compute the name rather than literally
            // using "self", as "self" is a symlink.
            let proc_self = openat(&proc, DecInt::new(pid.as_raw()), oflags, Mode::empty())
                .map_err(|_err| io::Error::NOTSUP)?;
            let proc_self_stat = check_proc_entry(
                Kind::Pid,
                proc_self.as_fd(),
                Some(proc_stat),
                uid.as_raw(),
                gid.as_raw(),
            )
            .map_err(|_err| io::Error::NOTSUP)?;

            Ok(new_static_fd(proc_self, proc_self_stat))
        })
        .map(|(owned, stat)| (owned.as_fd(), stat))
}

/// Returns a handle to Linux's `/proc/self/fd` directory.
///
/// This ensures that `/proc/self/fd` is `procfs`, that nothing is mounted on
/// top of it, and that it looks normal.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man5/proc.5.html
pub fn proc_self_fd() -> io::Result<BorrowedFd<'static>> {
    static PROC_SELF_FD: StaticFd = StaticFd::new();

    // The init function here may run multiple times; see above.
    PROC_SELF_FD
        .get_or_try_init(|| {
            let (_, proc_stat) = proc()?;

            let (proc_self, proc_self_stat) = proc_self()?;
            let oflags = OFlags::NOFOLLOW
                | OFlags::PATH
                | OFlags::DIRECTORY
                | OFlags::CLOEXEC
                | OFlags::NOCTTY
                | OFlags::NOATIME;

            // Open "/proc/self/fd".
            let proc_self_fd = openat(&proc_self, cstr!("fd"), oflags, Mode::empty())
                .map_err(|_err| io::Error::NOTSUP)?;
            let proc_self_fd_stat = check_proc_entry(
                Kind::Fd,
                proc_self_fd.as_fd(),
                Some(proc_stat),
                proc_self_stat.st_uid,
                proc_self_stat.st_gid,
            )
            .map_err(|_err| io::Error::NOTSUP)?;

            Ok(new_static_fd(proc_self_fd, proc_self_fd_stat))
        })
        .map(|(owned, _stat)| owned.as_fd())
}

#[cfg(feature = "std")]
type StaticFd = OnceCell<(OwnedFd, Stat)>;
#[cfg(feature = "no_std_demo")]
type StaticFd = OnceBox<(OwnedFd, Stat)>;

#[cfg(feature = "std")]
#[inline]
fn new_static_fd(fd: OwnedFd, stat: Stat) -> (OwnedFd, Stat) {
    (fd, stat)
}
#[cfg(feature = "no_std_demo")]
#[inline]
fn new_static_fd(fd: OwnedFd, stat: Stat) -> Box<(OwnedFd, Stat)> {
    Box::new((fd, stat))
}
