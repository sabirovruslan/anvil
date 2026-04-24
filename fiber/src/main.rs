use std::{
    arch::{asm, naked_asm},
    collections::VecDeque,
};

const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 2;
const MAX_THREADS: usize = 4;

// only for example
static mut RUNTIME: *mut Runtime = std::ptr::null_mut();

#[derive(PartialEq, Eq, Debug)]
enum State {
    Available,
    Running,
    Ready,
}

#[derive(Default, Debug)]
#[repr(C)]
struct ThreadContext {
    sp: u64,
    x19: u64,
    x20: u64,
    x21: u64,
    x22: u64,
    x23: u64,
    x24: u64,
    x25: u64,
    x26: u64,
    x27: u64,
    x28: u64,
    x29: u64, // frame pointer
    x30: u64, // lr
}

struct Thread {
    stack: Vec<u8>,
    ctx: ThreadContext,
    state: State,
}

pub struct Runtime {
    threads: Vec<Thread>,
    current: usize,
    tasks: VecDeque<fn()>,
}

impl Thread {
    fn new() -> Self {
        Self {
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Available,
        }
    }
}

impl Runtime {
    pub fn new() -> Self {
        let base_thread = Thread {
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Running,
        };

        let mut threads = vec![base_thread];
        let mut available_threads = (1..MAX_THREADS).map(|_| Thread::new()).collect();
        threads.append(&mut available_threads);

        Self {
            threads,
            current: 0,
            tasks: VecDeque::new(),
        }
    }
    pub fn init(&mut self) {
        unsafe {
            RUNTIME = self as *mut Runtime;
        }
    }

    pub fn run(&mut self) -> ! {
        while self.t_yield() {
            self.fill_available_threads();
        }

        std::process::exit(0);
    }

    fn t_return(&mut self) {
        if self.current != 0 {
            self.threads[self.current].state = State::Available;
            self.t_yield();
        }
    }

    fn fill_available_threads(&mut self) {
        for i in 1..self.threads.len() {
            if self.threads[i].state == State::Available
                && let Some(task) = self.tasks.pop_front()
            {
                self.schedule_task(i, task);
            }
        }
    }

    #[inline(never)]
    fn t_yield(&mut self) -> bool {
        let mut pos = self.current;
        while self.threads[pos].state != State::Ready {
            pos += 1;
            if pos == self.threads.len() {
                pos = 0;
            }

            if pos == self.current {
                return false;
            }
        }

        if self.threads[self.current].state != State::Available {
            self.threads[self.current].state = State::Ready;
        }

        let prev_pos = self.current;
        self.current = pos;
        self.threads[self.current].state = State::Running;

        unsafe {
            let prev_ctx: *mut ThreadContext = &mut self.threads[prev_pos].ctx;
            let next_ctx: *const ThreadContext = &self.threads[pos].ctx;
            asm!(
                "bl {switch_fn}",
                switch_fn = sym switch,
                in("x0") prev_ctx,
                in("x1") next_ctx,
                clobber_abi("C")
            );
        }

        true
    }

    pub fn spawn(&mut self, f: fn()) {
        let idx_thread = self
            .threads
            .iter()
            .position(|t| t.state == State::Available);

        if idx_thread.is_none() {
            self.tasks.push_back(f);
            return;
        }

        self.schedule_task(idx_thread.unwrap(), f);
    }

    fn schedule_task(&mut self, index: usize, f: fn()) {
        let available_thread = &mut self.threads[index];
        let size = available_thread.stack.len();

        unsafe {
            let sp = available_thread.stack.as_mut_ptr().add(size);
            let sp = (sp as usize & !15) as *mut u8;

            available_thread.ctx.sp = sp as u64;
            available_thread.ctx.x19 = f as usize as u64;
            available_thread.ctx.x20 = guard as usize as u64;
            available_thread.ctx.x30 = thread_entry as usize as u64;
        }

        available_thread.state = State::Ready;
    }
}

fn guard() {
    unsafe {
        (*RUNTIME).t_return();
    }
}

pub fn yield_thread() {
    unsafe {
        (*RUNTIME).t_yield();
    }
}

#[unsafe(naked)]
unsafe extern "C" fn thread_entry() {
    naked_asm!("mov x9, x19", "blr x9", "br x20",);
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn switch() {
    naked_asm!(
        // save old context into *prev_ctx (x0)
        "mov x9, sp",
        "str x9,  [x0, #0x00]",
        "str x19, [x0, #0x08]",
        "str x20, [x0, #0x10]",
        "str x21, [x0, #0x18]",
        "str x22, [x0, #0x20]",
        "str x23, [x0, #0x28]",
        "str x24, [x0, #0x30]",
        "str x25, [x0, #0x38]",
        "str x26, [x0, #0x40]",
        "str x27, [x0, #0x48]",
        "str x28, [x0, #0x50]",
        "str x29, [x0, #0x58]",
        "str x30, [x0, #0x60]",
        // load new context from *next_ctx (x1)
        "ldr x9,  [x1, #0x00]",
        "mov sp, x9",
        "ldr x19, [x1, #0x08]",
        "ldr x20, [x1, #0x10]",
        "ldr x21, [x1, #0x18]",
        "ldr x22, [x1, #0x20]",
        "ldr x23, [x1, #0x28]",
        "ldr x24, [x1, #0x30]",
        "ldr x25, [x1, #0x38]",
        "ldr x26, [x1, #0x40]",
        "ldr x27, [x1, #0x48]",
        "ldr x28, [x1, #0x50]",
        "ldr x29, [x1, #0x58]",
        "ldr x30, [x1, #0x60]",
        // continue execution of the restored context
        "ret"
    );
}

fn main() {
    let mut runtime = Runtime::new();
    runtime.init();

    runtime.spawn(|| {
        let id = 1;

        for i in 0..10 {
            println!("thread: {}, cnt: {}", id, i);
            yield_thread();
        }
        println!("thread {} finished", id);
    });

    runtime.spawn(|| {
        let id = 2;

        for i in 0..5 {
            println!("thread: {}, cnt: {}", id, i);
            yield_thread();
        }
        println!("thread {} finished", id);
    });

    runtime.spawn(|| {
        let id = 3;

        for i in 0..3 {
            println!("thread: {}, cnt: {}", id, i);
            yield_thread();
        }
        println!("thread {} finished", id);
    });

    runtime.spawn(|| {
        let id = 4;

        for i in 0..3 {
            println!("thread: {}, cnt: {}", id, i);
            yield_thread();
        }
        println!("thread {} finished", id);
    });

    runtime.spawn(|| {
        let id = 5;

        for i in 0..2 {
            println!("thread: {}, cnt: {}", id, i);
            yield_thread();
        }
        println!("thread {} finished", id);
    });

    runtime.run();
}
