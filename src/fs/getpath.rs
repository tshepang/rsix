use crate::std_path::PathBuf;
use crate::{imp, io};
use io_lifetimes::AsFd;

/// `fcntl(fd, F_GETPATH)`
///
/// # References
///  - [Apple]
///
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fcntl.2.html
#[inline]
pub fn getpath<Fd: AsFd>(fd: &Fd) -> io::Result<PathBuf> {
    let fd = fd.as_fd();
    imp::syscalls::getpath(fd)
}
