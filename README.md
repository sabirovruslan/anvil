# anvil

Low-level Rust experiments for understanding how concurrency, async runtimes, fibers, context switching, and event-driven I/O work under the hood.

This repository is a learning and portfolio project focused on rebuilding the core primitives behind asynchronous systems instead of only using high-level libraries. The goal is to develop a system-level understanding of OS threads, user-space scheduling, manual stack switching, polling, and the mechanics that eventually lead to `Future` and executor-based runtimes.

## Learning Context

This repository is being developed while studying *Asynchronous Programming in Rust* by Carl Fredrik Samson.

The book provides part of the foundation, but the goal of this project is broader: use those ideas to build additional experiments and a deeper system-level understanding of concurrency, fibers, polling, and async runtime internals.

The repository is designed to answer questions like:

- What is the practical difference between threads, fibers, and async tasks?
- What does a context switch look like at the register and stack level?
- How can a runtime schedule work in user space?
- How does event polling fit into an async runtime?
- What low-level pieces are required before `async/await` becomes possible?

## Current Experiments

### `c_thread`

Explores thread-based concurrency and output through both libc FFI and direct syscall-style assembly. This crate is useful for comparing higher-level Rust threading with lower-level execution details.

Run:

```bash
cargo run -p c_thread
```

### `stack-swap`

Demonstrates how execution can continue on a manually prepared stack. This is one of the core ideas behind fibers, coroutines, and runtime-managed tasks.

Run:

```bash
cargo run -p stack-swap
```

### `fiber`

Implements a small cooperative scheduler with its own thread contexts, stacks, and yielding mechanism. This crate is the clearest bridge between low-level context switching and higher-level runtime design.

Run:

```bash
cargo run -p fiber
```

### `k_queue`

Explores event-driven I/O using `kqueue`, which is one of the OS primitives used to build scalable non-blocking systems and reactor-style runtimes.

Run:

```bash
cargo run -p k_queue
```

## Platform Notes

- The current repository is being explored on `Darwin arm64`.
- Some inline assembly and calling-convention details are architecture-specific.
- `kqueue` is relevant to macOS and BSD systems; a future Linux-oriented counterpart would likely use `epoll`.
- These experiments are intentionally minimal and educational, not production-ready runtime implementations.

## Portfolio Intent

This repository is meant to show system-level reasoning in Rust, especially around concurrency and async internals. It is less about framework usage and more about understanding the mechanisms that frameworks and runtimes depend on.
