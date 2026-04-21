use std::arch::asm;

const SSIZE: usize = 1024 * 128;

#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    #[cfg(target_arch = "x86_64")]
    rsp: u64,

    #[cfg(target_arch = "aarch64")]
    sp: u64,

    #[cfg(target_arch = "aarch64")]
    lr: u64,

    #[cfg(target_arch = "aarch64")]
    fp: u64,
}

fn hello() -> ! {
    println!("WAKING UP ON A NEW STACK (ARM64)");
    loop {}
}

#[cfg(target_arch = "aarch64")]
unsafe fn gt_switch(new: *const ThreadContext) {
    unsafe {
        asm!(
            "ldr x0, [{0}, #0]",
            "ldr x1, [{0}, #8]",
            "ldr x2, [{0}, #16]",

            "mov sp, x0",
            "mov lr, x1",
            "mov fp, x2",

            "ret",
            in(reg) new,
            out("x0") _,
            out("x1") _,
            out("x2") _
        );
    }
}

#[cfg(target_arch = "x86_64")]
unsafe fn gt_switch(new: *const ThreadContext) {
    unsafe {
        asm!(
            "mov rsp, [{0} + 0x00]",
            "ret",
            in(reg) new,
        );
    }
}

fn main() {
    let mut ctx = ThreadContext::default();
    let mut stack = vec![0_u8; SSIZE];

    unsafe {
        let stack_bottom = stack.as_mut_ptr().add(SSIZE);
        let stack_bottom_aligned = (stack_bottom as usize & !15) as *mut u8;

        #[cfg(target_arch = "aarch64")]
        {
            let sp = stack_bottom_aligned as u64;

            ctx.sp = sp;
            ctx.lr = hello as u64;
            ctx.fp = 0;
        }

        #[cfg(target_arch = "x86_64")]
        {
            let sp = stack_bottom_aligned.offset(-16) as u64;
            std::ptr::write(sp as *mut u64, hello as u64);
            ctx.rsp = sp;
        }

        println!("Switching to new stack...");

        gt_switch(&ctx);
    }
}
