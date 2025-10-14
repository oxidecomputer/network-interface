use std::mem;
use crate::{Error, Result};

pub struct IfAddrIterator {
    base: *mut libc::ifaddrs,
    next: *mut libc::ifaddrs,
}

impl Iterator for IfAddrIterator {
    type Item = *mut libc::ifaddrs;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        }

        let current_next = self.next;
        let next_next = unsafe { (*self.next).ifa_next };
        self.next = next_next;
        Some(current_next)
    }
}

impl Drop for IfAddrIterator {
    fn drop(&mut self) {
        unsafe { libc::freeifaddrs(self.base) }
    }
}

pub fn getifaddrs() -> Result<IfAddrIterator> {
    let mut addr = mem::MaybeUninit::<*mut libc::ifaddrs>::uninit();
    match unsafe { libc::getifaddrs(addr.as_mut_ptr()) } {
        0 => Ok(IfAddrIterator {
            base: unsafe { addr.assume_init() },
            next: unsafe { addr.assume_init() },
        }),
        getifaddrs_result => Err(Error::GetIfAddrsError(
            String::from("getifaddrs"),
            getifaddrs_result,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_ok() {
        let empty = IfAddrIterator {
            base: std::ptr::null_mut(),
            next: std::ptr::null_mut(),
        };

        assert_eq!(empty.count(), 0);
    }

    #[test]
    fn test_iter_has_all_ifa() {
        // Get the number of interfaces returned by the `IfAddrIterator` wrapper.
        let iter_ct = getifaddrs().unwrap().count();

        // Manually get the number of interfaces returned by libc.
        let mut raw_ct = 0;
        let mut ifap = mem::MaybeUninit::<*mut libc::ifaddrs>::uninit();
        let ret = unsafe { libc::getifaddrs(ifap.as_mut_ptr()) };

        assert_eq!(ret, 0);
        let ifap = unsafe { ifap.assume_init() };

        if !ifap.is_null() {
            raw_ct += 1;
        }
        let mut next = unsafe { (*ifap).ifa_next };
        while !next.is_null() {
            raw_ct += 1;
            next = unsafe { (*next).ifa_next };
        }
        unsafe { libc::freeifaddrs(ifap) };

        assert_eq!(iter_ct, raw_ct);
    }
}
