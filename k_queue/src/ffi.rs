pub const EV_ADD: u16 = 0x0001;
pub const EVFILT_READ: i16 = -1;
pub const EV_CLEAR: u16 = 0x0020;

#[link(name = "c")]
unsafe extern "C" {
    pub unsafe fn kqueue() -> i32;
    pub unsafe fn close(fd: i32) -> i32;
    pub unsafe fn kevent(
        kq: i32,
        changelist: *const Event,
        nchanges: i32,
        eventlist: *mut Event,
        nevents: i32,
        timeout: *const timespec,
    ) -> i32;
}

#[derive(Debug)]
#[repr(C)]
#[cfg(target_arch = "aarch64")]
#[repr(packed)]
pub struct Event {
    pub(crate) ident: usize,
    pub(crate) filter: i16,
    pub(crate) flags: u16,
    pub(crate) fflags: u32,
    pub(crate) data: isize,
    pub(crate) udata: usize,
}

impl Event {
    pub fn token(&self) -> usize {
        self.udata
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct timespec {
    pub(crate) tv_sec: isize,
    pub(crate) tv_nsec: isize,
}
