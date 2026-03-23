use std::{
    arch::asm,
    io::{Error, Result},
    thread::{self, sleep},
    time::Duration,
};

#[link(name = "c")]
unsafe extern "C" {
    unsafe fn write(fd: u32, buf: *const u8, count: usize) -> i32;
}

fn ffi_syscall(message: String) -> Result<()> {
    let msg_ptr = message.as_ptr();
    let len = message.len();

    let res = unsafe { write(1, msg_ptr, len) };

    if res == -1 {
        return Err(Error::last_os_error());
    }

    Ok(())
}

#[inline(never)]
fn raw_syscall(message: String) {
    let msp_ptr = message.as_ptr();
    let len = message.len();

    unsafe {
        asm!(
            "mov x16, 4",
            "mov x0, 1",
            "svc 0",
            in("x1") msp_ptr,
            in("x2") len,
            out("x16") _,
            out("x0") _,
            lateout("x1") _,
            lateout("x2") _
        )
    }
}

fn main() {
    raw_syscall("Start program here!\n".to_string());

    let t1 = thread::spawn(move || {
        sleep(Duration::from_millis(200));
        raw_syscall("The long running task finish last \n".to_string());
    });

    let t2 = thread::spawn(move || {
        sleep(Duration::from_millis(100));
        ffi_syscall("We can chain callback ...\n".to_string()).unwrap();

        let t3 = thread::spawn(move || {
            sleep(Duration::from_millis(50));
            println!("... like this!");
        });

        t3.join().unwrap();
    });

    t1.join().unwrap();
    t2.join().unwrap();
}
