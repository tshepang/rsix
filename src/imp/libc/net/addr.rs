//! IPv4, IPv6, and Socket addresses.

use super::AddressFamily;
use crate::std_ffi::CString;
use crate::{io, path};
use core::fmt;
#[cfg(any(
    target_os = "netbsd",
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd"
))]
use core::mem::size_of;

/// `struct in_addr`
#[repr(transparent)]
#[derive(Clone)]
#[doc(alias = "in_addr")]
pub struct Ipv4Addr(pub(crate) libc::in_addr);

impl Ipv4Addr {
    #[cfg(feature = "std")]
    pub const BROADCAST: Self = Self::from_std(crate::std_net::Ipv4Addr::BROADCAST);
    #[cfg(feature = "std")]
    pub const LOCALHOST: Self = Self::from_std(crate::std_net::Ipv4Addr::LOCALHOST);
    #[cfg(feature = "std")]
    pub const UNSPECIFIED: Self = Self::from_std(crate::std_net::Ipv4Addr::UNSPECIFIED);

    /// Construct a new IPv4 address from 4 octets.
    #[inline]
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self(libc::in_addr {
            s_addr: u32::from_ne_bytes([a, b, c, d]),
        })
    }

    #[inline]
    pub const fn from_std(std: crate::std_net::Ipv4Addr) -> Self {
        let raw: u32 = u32::from_be_bytes(std.octets());
        Self(libc::in_addr {
            s_addr: raw.to_be(),
        })
    }

    #[inline]
    pub const fn into_std(self) -> crate::std_net::Ipv4Addr {
        let octets = self.0.s_addr.to_ne_bytes();
        crate::std_net::Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3])
    }

    #[inline]
    pub const fn is_unspecified(&self) -> bool {
        self.const_clone().into_std().is_unspecified()
    }

    #[inline]
    pub const fn is_loopback(&self) -> bool {
        self.const_clone().into_std().is_loopback()
    }

    #[inline]
    pub const fn is_private(&self) -> bool {
        self.const_clone().into_std().is_private()
    }

    #[inline]
    pub const fn is_link_local(&self) -> bool {
        self.const_clone().into_std().is_link_local()
    }

    #[inline]
    pub const fn is_multicast(&self) -> bool {
        self.const_clone().into_std().is_multicast()
    }

    #[inline]
    pub const fn is_broadcast(&self) -> bool {
        self.const_clone().into_std().is_broadcast()
    }

    #[inline]
    pub const fn is_documentation(&self) -> bool {
        self.const_clone().into_std().is_documentation()
    }

    #[inline]
    pub const fn to_ipv6_compatible(&self) -> Ipv6Addr {
        Ipv6Addr::from_std(self.const_clone().into_std().to_ipv6_compatible())
    }

    #[inline]
    pub const fn to_ipv6_mapped(&self) -> Ipv6Addr {
        Ipv6Addr::from_std(self.const_clone().into_std().to_ipv6_mapped())
    }

    #[inline]
    pub const fn octets(&self) -> [u8; 4] {
        // `s_addr` is already in big-endian format.
        self.0.s_addr.to_ne_bytes()
    }

    #[inline]
    const fn const_clone(&self) -> Self {
        Self(self.0)
    }
}

impl fmt::Display for Ipv4Addr {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.const_clone().into_std().fmt(fmt)
    }
}

impl fmt::Debug for Ipv4Addr {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

/// `struct in6_addr`
#[repr(transparent)]
#[derive(Clone)]
#[doc(alias = "in6_addr")]
pub struct Ipv6Addr(pub(crate) libc::in6_addr);

impl Ipv6Addr {
    #[cfg(feature = "std")]
    pub const LOCALHOST: Self = Self::from_std(crate::std_net::Ipv6Addr::LOCALHOST);
    #[cfg(feature = "std")]
    pub const UNSPECIFIED: Self = Self::from_std(crate::std_net::Ipv6Addr::UNSPECIFIED);

    /// Construct a new IPv6 address from eight 16-bit segments.
    #[allow(clippy::many_single_char_names, clippy::too_many_arguments)]
    pub const fn new(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> Self {
        Self(libc::in6_addr {
            s6_addr: [
                (a >> 8) as u8,
                (a & 0xff) as u8,
                (b >> 8) as u8,
                (b & 0xff) as u8,
                (c >> 8) as u8,
                (c & 0xff) as u8,
                (d >> 8) as u8,
                (d & 0xff) as u8,
                (e >> 8) as u8,
                (e & 0xff) as u8,
                (f >> 8) as u8,
                (f & 0xff) as u8,
                (g >> 8) as u8,
                (g & 0xff) as u8,
                (h >> 8) as u8,
                (h & 0xff) as u8,
            ],
        })
    }

    #[inline]
    pub const fn from_std(std: crate::std_net::Ipv6Addr) -> Self {
        Self(libc::in6_addr {
            s6_addr: std.octets(),
        })
    }

    #[inline]
    pub const fn into_std(self) -> crate::std_net::Ipv6Addr {
        let segments = self.segments();
        crate::std_net::Ipv6Addr::new(
            segments[0],
            segments[1],
            segments[2],
            segments[3],
            segments[4],
            segments[5],
            segments[6],
            segments[7],
        )
    }

    #[inline]
    pub const fn is_unspecified(&self) -> bool {
        self.const_clone().into_std().is_unspecified()
    }

    #[inline]
    pub const fn is_loopback(&self) -> bool {
        self.const_clone().into_std().is_loopback()
    }

    #[inline]
    pub const fn to_ipv4(&self) -> Option<Ipv4Addr> {
        match self.const_clone().into_std().to_ipv4() {
            None => None,
            Some(ipv4) => Some(Ipv4Addr::from_std(ipv4)),
        }
    }

    #[inline]
    pub const fn octets(&self) -> [u8; 16] {
        self.0.s6_addr
    }

    #[inline]
    pub const fn segments(&self) -> [u16; 8] {
        [
            u16::from_be_bytes([self.0.s6_addr[0], self.0.s6_addr[1]]),
            u16::from_be_bytes([self.0.s6_addr[2], self.0.s6_addr[3]]),
            u16::from_be_bytes([self.0.s6_addr[4], self.0.s6_addr[5]]),
            u16::from_be_bytes([self.0.s6_addr[6], self.0.s6_addr[7]]),
            u16::from_be_bytes([self.0.s6_addr[8], self.0.s6_addr[9]]),
            u16::from_be_bytes([self.0.s6_addr[10], self.0.s6_addr[11]]),
            u16::from_be_bytes([self.0.s6_addr[12], self.0.s6_addr[13]]),
            u16::from_be_bytes([self.0.s6_addr[14], self.0.s6_addr[15]]),
        ]
    }

    #[inline]
    const fn const_clone(&self) -> Self {
        Self(self.0)
    }
}

impl fmt::Display for Ipv6Addr {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.const_clone().into_std().fmt(fmt)
    }
}

impl fmt::Debug for Ipv6Addr {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

/// `struct sockaddr_in`
#[derive(Clone)]
#[doc(alias = "sockaddr_in")]
pub struct SocketAddrV4 {
    pub(crate) addr: Ipv4Addr,
    pub(crate) port: u16,
}

impl SocketAddrV4 {
    /// Construct a new IPv4 socket address from an address and a port.
    #[inline]
    pub const fn new(addr: Ipv4Addr, port: u16) -> Self {
        Self { addr, port }
    }

    /// Encode this socket address in the host format.
    #[inline]
    pub(crate) const fn encode(&self) -> libc::sockaddr_in {
        libc::sockaddr_in {
            #[cfg(any(
                target_os = "netbsd",
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "openbsd"
            ))]
            sin_len: size_of::<libc::sockaddr_in>() as _,
            sin_family: libc::AF_INET as _,
            sin_addr: self.addr.0,
            sin_port: self.port.to_be(),
            sin_zero: [0; 8_usize],
        }
    }

    /// Return the IPv4 address of this socket address.
    #[inline]
    pub const fn address(&self) -> &Ipv4Addr {
        &self.addr
    }

    /// Return the port of this address.
    #[inline]
    pub const fn port(&self) -> u16 {
        self.port
    }
}

impl fmt::Display for SocketAddrV4 {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        crate::std_net::SocketAddrV4::new(self.address().const_clone().into_std(), self.port())
            .fmt(fmt)
    }
}

impl fmt::Debug for SocketAddrV4 {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

/// `struct sockaddr_in6`
#[derive(Clone)]
#[doc(alias = "sockaddr_in6")]
pub struct SocketAddrV6 {
    pub(crate) addr: Ipv6Addr,
    pub(crate) port: u16,
    pub(crate) flowinfo: u32,
    pub(crate) scope_id: u32,
}

impl SocketAddrV6 {
    /// Construct a new IPv6 socket address from an address, port, flow info,
    /// and scope id.
    #[inline]
    pub const fn new(addr: Ipv6Addr, port: u16, flowinfo: u32, scope_id: u32) -> Self {
        Self {
            addr,
            port,
            flowinfo,
            scope_id,
        }
    }

    /// Encode this socket address in the host format.
    #[inline]
    pub(crate) const fn encode(&self) -> libc::sockaddr_in6 {
        libc::sockaddr_in6 {
            #[cfg(any(
                target_os = "netbsd",
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "openbsd"
            ))]
            sin6_len: size_of::<libc::sockaddr_in6>() as _,
            sin6_family: libc::AF_INET6 as _,
            sin6_addr: self.addr.0,
            sin6_port: self.port.to_be(),
            sin6_flowinfo: self.flowinfo,
            sin6_scope_id: self.scope_id,
        }
    }

    /// Return the IPv6 address of this socket address.
    #[inline]
    pub const fn address(&self) -> &Ipv6Addr {
        &self.addr
    }

    /// Return the port of this address.
    #[inline]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Return the flowinfo of this address.
    #[inline]
    pub const fn flowinfo(&self) -> u32 {
        self.flowinfo
    }

    /// Return the scope_id of this address.
    #[inline]
    pub const fn scope_id(&self) -> u32 {
        self.scope_id
    }
}

impl fmt::Display for SocketAddrV6 {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        crate::std_net::SocketAddrV6::new(
            self.address().const_clone().into_std(),
            self.port(),
            self.flowinfo(),
            self.scope_id(),
        )
        .fmt(fmt)
    }
}

impl fmt::Debug for SocketAddrV6 {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

/// `struct sockaddr_un`
#[derive(Clone)]
#[doc(alias = "sockaddr_un")]
pub struct SocketAddrUnix {
    path: CString,
}

impl SocketAddrUnix {
    /// Construct a new Unix-domain address from a byte slice.
    /// filesystem path.
    #[inline]
    pub fn new<P: path::Arg>(path: P) -> io::Result<Self> {
        let path = path.into_c_str()?.into_owned();
        Self::_new(path)
    }

    #[inline]
    fn _new(path: CString) -> io::Result<Self> {
        let bytes = path.as_bytes();

        let z = libc::sockaddr_un {
            #[cfg(any(
                target_os = "netbsd",
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "openbsd"
            ))]
            sun_len: 0,
            sun_family: 0,
            #[cfg(any(
                target_os = "netbsd",
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "openbsd"
            ))]
            sun_path: [0; 104],
            #[cfg(not(any(
                target_os = "netbsd",
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "openbsd"
            )))]
            sun_path: [0; 108],
        };
        if bytes.len() + 1 > z.sun_path.len() {
            return Err(io::Error::NAMETOOLONG);
        }
        Ok(Self { path })
    }

    /// Encode this socket address in the host format.
    #[inline]
    pub(crate) fn encode(&self) -> libc::sockaddr_un {
        let mut encoded = libc::sockaddr_un {
            #[cfg(any(
                target_os = "netbsd",
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "openbsd"
            ))]
            sun_len: size_of::<libc::sockaddr_un>() as _,
            sun_family: libc::AF_UNIX as _,
            #[cfg(any(
                target_os = "netbsd",
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "openbsd"
            ))]
            sun_path: [0; 104],
            #[cfg(not(any(
                target_os = "netbsd",
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "openbsd"
            )))]
            sun_path: [0; 108],
        };
        let bytes = self.path.as_bytes();
        for (i, b) in bytes.iter().enumerate() {
            encoded.sun_path[i] = *b as libc::c_char;
        }
        encoded.sun_path[bytes.len()] = b'\0' as libc::c_char;
        encoded
    }
}

impl fmt::Debug for SocketAddrUnix {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.path.fmt(fmt)
    }
}

/// `struct sockaddr_storage`
#[derive(Clone)]
#[doc(alias = "sockaddr")]
#[non_exhaustive]
pub enum SocketAddr {
    /// `struct sockaddr_in`
    V4(SocketAddrV4),
    /// `struct sockaddr_in6`
    V6(SocketAddrV6),
    /// `struct sockaddr_un`
    Unix(SocketAddrUnix),
}

impl SocketAddr {
    /// Return the address family of this socket address.
    #[inline]
    pub const fn address_family(&self) -> AddressFamily {
        match self {
            SocketAddr::V4(_) => AddressFamily::INET,
            SocketAddr::V6(_) => AddressFamily::INET6,
            SocketAddr::Unix(_) => AddressFamily::UNIX,
        }
    }
}

impl fmt::Debug for SocketAddr {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SocketAddr::V4(v4) => v4.fmt(fmt),
            SocketAddr::V6(v6) => v6.fmt(fmt),
            SocketAddr::Unix(unix) => unix.fmt(fmt),
        }
    }
}
