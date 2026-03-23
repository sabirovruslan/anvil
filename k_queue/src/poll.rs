use std::{
    io::{self, Result},
    net::TcpStream,
    os::fd::AsRawFd,
    ptr,
};

use crate::ffi::{self, EV_CLEAR, Event, timespec};

type Events = Vec<ffi::Event>;

pub struct Poll {
    registry: Registry,
}

impl Poll {
    pub fn new() -> Result<Self> {
        let res = unsafe { ffi::kqueue() };

        if res < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Self {
            registry: Registry { raw_fd: res },
        })
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn poll(&mut self, events: &mut Events, timeout: Option<i32>) -> Result<()> {
        let kq = self.registry.raw_fd;
        let timeout_ptr = match timeout {
            None | Some(-1) => ptr::null(),
            Some(t) if t >= 0 => {
                let ts = ffi::timespec {
                    tv_sec: t as isize,
                    tv_nsec: 0,
                };
                &ts as *const ffi::timespec
            }
            Some(_) => ptr::null(),
        };
        let max_events = events.capacity() as i32;
        let res = unsafe {
            ffi::kevent(
                kq,
                ptr::null(),
                0,
                events.as_mut_ptr(),
                max_events,
                timeout_ptr,
            )
        };

        if res < 0 {
            return Err(io::Error::last_os_error());
        }

        unsafe {
            events.set_len(res as usize);
        }

        Ok(())
    }
}

pub struct Registry {
    raw_fd: i32,
}

impl Registry {
    pub fn register(&self, source: &TcpStream, token: usize, interests: i32) -> Result<()> {
        let mut change = ffi::Event {
            ident: source.as_raw_fd() as usize,
            filter: ffi::EVFILT_READ,
            flags: interests as u16,
            fflags: 0,
            data: 0,
            udata: token,
        };

        let res = unsafe {
            ffi::kevent(
                self.raw_fd,
                &change as *const _,
                1,
                ptr::null_mut(),
                0,
                ptr::null(),
            )
        };

        if res < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        let res = unsafe { ffi::close(self.raw_fd) };

        if res < 0 {
            let err = io::Error::last_os_error();
            println!("ERROR: {err:?}");
        }
    }
}
