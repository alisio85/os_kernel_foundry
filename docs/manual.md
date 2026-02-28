# os_kernel_foundry Manual

This manual is a detailed guide to using **os_kernel_foundry** as the foundation of your own Rust-based operating system.

The crate is intentionally a **foundation** library:

- It provides **traits** and small **orchestration helpers**.
- It does **not** provide a runnable kernel binary, bootloader integration, linker scripts, or architecture-specific low-level code.

If you want a full OS that boots on real hardware, you typically create a *separate* kernel workspace that depends on this crate.

---

## Table of contents

- [1. Mental model](#1-mental-model)
- [2. Crate integration](#2-crate-integration)
- [3. Architecture back-ends (`arch`)](#3-architecture-back-ends-arch)
- [4. Boot pipelines (`boot`)](#4-boot-pipelines-boot)
- [5. Kernel orchestration (`kernel`)](#5-kernel-orchestration-kernel)
- [6. Devices (`device`)](#6-devices-device)
- [7. Memory (`memory`)](#7-memory-memory)
- [8. Synchronization (`sync`)](#8-synchronization-sync)
- [9. Scheduling (`scheduler`)](#9-scheduling-scheduler)
- [10. IPC (`ipc`)](#10-ipc-ipc)
- [11. Testing strategy](#11-testing-strategy)
- [12. `no_std` usage patterns](#12-no_std-usage-patterns)
- [13. Common pitfalls](#13-common-pitfalls)

---

## 1. Mental model

The crate is designed around an explicit separation of concerns.

- **Portable kernel logic** should depend only on *traits*.
- **Architecture/platform-specific logic** is implemented by you, outside this crate.
- **Boot** is modeled as a deterministic, testable pipeline of stages.

At a high level, you typically build your kernel like this:

1. Choose an architecture type `A` that implements `arch::Architecture`.
2. Build a list of boot stages `&[&dyn boot::BootStage<A>]`.
3. Construct `kernel::Kernel<A>` and call `Kernel::boot`.
4. Initialize devices via `device::DeviceRegistry`.
5. Continue into your scheduler / IPC / other subsystems.

Important: `os_kernel_foundry` gives you **boundaries** and **contracts**, not a full OS.

---

## 2. Crate integration

### 2.1 Add the dependency

**Via path** (for developing both crates together):

```toml
[dependencies]
os_kernel_foundry = { path = "../os_kernel_foundry" }
```

**Via crates.io** (when published):

```toml
[dependencies]
os_kernel_foundry = "0.1"
```

### 2.2 Features

- `std`: enables `std` support for host-side experimentation and unit tests.

The crate is `no_std` outside tests via `#![cfg_attr(not(test), no_std)]`.

As a consumer, you generally:

- keep your kernel `#![no_std]`
- enable `std` only in host-side tests or simulator binaries

---

## 3. Architecture back-ends (`arch`)

The `arch` module defines the minimal services your portable kernel logic expects.

### 3.1 `Timer`

```rust
pub trait Timer {
    type Tick: Copy + Ord;
    fn now(&self) -> Self::Tick;
}
```

**Design intent**

- Must be monotonic.
- Only exposes a tick counter; conversion to durations is left to higher layers.

**Typical real back-ends**

- x86_64: TSC / HPET / APIC timer
- aarch64: generic timer

**Typical test back-end**

- deterministic in-memory timer you can advance manually

### 3.2 `InterruptController`

```rust
pub trait InterruptController {
    fn enable(&mut self, id: u32);
    fn disable(&mut self, id: u32);
    fn acknowledge(&mut self, id: u32);
}
```

**Design intent**

- Explicitly models a small subset used during early kernel work.
- Keeps portable code independent from GIC/APIC/PLIC details.

### 3.3 `AddressTranslator`

```rust
pub trait AddressTranslator {
    fn translate(&self, virtual_address: usize) -> Option<usize>;
}
```

**Design intent**

- Gives portable code a minimal “best effort” translation operation.
- You can implement this using a page table walker, a mapping database, or a hypervisor API.

### 3.4 `Architecture`

```rust
pub trait Architecture {
    type Timer: Timer;
    type InterruptController: InterruptController;
    type AddressTranslator: AddressTranslator;

    fn timer(&self) -> &Self::Timer;
    fn interrupt_controller(&mut self) -> &mut Self::InterruptController;
    fn address_translator(&self) -> &Self::AddressTranslator;
}
```

**Key point**

Your kernel orchestration is parameterized over `A: Architecture`, which makes it easy to:

- swap architecture back-ends without changing portable logic
- test everything on the host with in-memory mocks

### 3.5 Recommended consumer layout

In *your* kernel repository, keep architecture back-ends separate:

- `kernel-core/` portable logic; depends on `os_kernel_foundry`
- `kernel-arch-x86_64/` implements `Architecture` for x86_64
- `kernel-arch-aarch64/` implements `Architecture` for aarch64
- `kernel-bin/` actual bootable binary crate (entrypoint, linker script, bootloader)

---

## 4. Boot pipelines (`boot`)

Boot is modeled as a sequence of small, focused stages.

### 4.1 The key types

- `BootStage<A>`: one stage
- `BootContext<'_, A>`: passed to stages; provides controlled access to the architecture
- `BootState`: progress state machine
- `BootError`: stage error

Simplified signatures:

```rust
pub trait BootStage<A: Architecture> {
    fn name(&self) -> &'static str;
    fn run(&self, ctx: &mut BootContext<'_, A>) -> Result<(), BootError>;
}

pub fn run_boot_sequence<A: Architecture>(
    arch: &mut A,
    stages: &[&dyn BootStage<A>],
) -> Result<BootState, BootError>;
```

### 4.2 Stage design guidelines

- Keep each stage **single-responsibility**.
- Make stages **idempotent** when practical.
- Treat stage boundaries as “checkpoints” for invariants.

Examples of stage responsibilities:

- configure timer (calibration, interrupt enable)
- install early exception/interrupt handlers
- initialize early physical allocator
- initialize paging (if you do that early)
- build a device registry
- bring up a scheduler

### 4.3 Passing state between stages

`BootContext` currently exposes:

- `ctx.arch()` to access and mutate the architecture
- `ctx.state()` to read the current `BootState`

If you need to pass additional data between stages (e.g. allocator instances, vmm handles, device registry), you typically do that in your own kernel crate by:

- storing state in your architecture type `A` (if it’s architecture-scoped), or
- storing state in a separate “kernel state” struct owned by your boot stages and referenced by them, or
- extending the abstraction in your fork (if you want the context to carry more state)

### 4.4 Boot errors

`BootError` is intentionally small:

- `BootError::Fatal(&'static str)`

In a real kernel, you might want richer errors (stage name, subsystem error types, error codes). A common approach is:

- keep a simple error type for early boot
- log details via a serial console
- then transition to richer error reporting later

---

## 5. Kernel orchestration (`kernel`)

The `kernel` module provides `Kernel<A>`, which binds everything to a single architecture instance.

### 5.1 Constructing and booting

Conceptually:

```rust
use os_kernel_foundry::kernel::Kernel;
use os_kernel_foundry::boot::BootStage;

fn boot_kernel<A: os_kernel_foundry::arch::Architecture>(
    arch: A,
    stages: &[&dyn BootStage<A>],
) {
    let mut kernel = Kernel::new(arch);
    let _boot_state = kernel.boot(stages);
}
```

Notes:

- `Kernel::boot` simply delegates to `boot::run_boot_sequence`.
- `Kernel` intentionally does **not** keep global state besides `arch`.

### 5.2 Device initialization

`Kernel::init_devices` takes a `DeviceRegistry` and calls `registry.init_all()`.

This keeps the orchestration simple and lets your kernel decide:

- which drivers exist
- which order they are registered
- when they should be initialized

---

## 6. Devices (`device`)

The device model is designed to work without heap allocation.

### 6.1 `DeviceDriver`

```rust
pub trait DeviceDriver {
    fn name(&self) -> &'static str;
    fn init(&mut self) -> Result<(), DeviceError>;
}
```

### 6.2 `DeviceRegistry`

The registry is fixed-capacity and backed by caller-provided storage:

```rust
pub struct DeviceRegistry<'a> {
    devices: &'a mut [Option<&'a mut dyn DeviceDriver>],
    count: usize,
}
```

**Usage pattern**

- Create storage as a fixed array.
- Create a registry from the storage.
- Register drivers.
- Initialize them.

Example skeleton:

```rust
use os_kernel_foundry::device::{DeviceDriver, DeviceRegistry};

struct UartDriver;
impl DeviceDriver for UartDriver {
    fn name(&self) -> &'static str { "uart" }
    fn init(&mut self) -> Result<(), os_kernel_foundry::device::DeviceError> { Ok(()) }
}

fn init_drivers() {
    let mut uart = UartDriver;

    let mut storage: [Option<&mut dyn DeviceDriver>; 8] = [
        None, None, None, None, None, None, None, None,
    ];

    let mut reg = DeviceRegistry::new(&mut storage);
    let ok = reg.register(&mut uart);
    assert!(ok);

    reg.init_all().unwrap();
}
```

**Why this design?**

- Early boot often has no allocator.
- Some kernels want strict control over memory layout.

---

## 7. Memory (`memory`)

The memory traits are intentionally minimal and `unsafe`.

### 7.1 `PhysicalMemoryAllocator`

```rust
pub trait PhysicalMemoryAllocator {
    unsafe fn allocate_frame(&mut self) -> Option<usize>;
    unsafe fn deallocate_frame(&mut self, frame: usize);
}
```

**Expected invariants (consumer responsibility)**

- A “frame” represents a unit of physical memory (e.g. 4 KiB page frame).
- Allocated frames must be tracked so they are:
  - not handed out twice simultaneously
  - not aliased in ways that break safety assumptions

The trait uses `usize` for maximum portability, but you can wrap it in your own strongly typed newtypes.

### 7.2 `VirtualMemoryManager`

```rust
pub trait VirtualMemoryManager {
    unsafe fn map(&mut self, virtual_address: usize, physical_address: usize) -> Result<(), ()>;
    unsafe fn unmap(&mut self, virtual_address: usize) -> Result<(), ()>;
}
```

**Design intent**

- Models the minimal “map/unmap” behavior.
- Does not impose paging structure.

**In a real kernel you will likely build a richer API** around this trait:

- typed virtual/physical address wrappers
- page size/permissions
- address space objects
- error types describing failures

---

## 8. Synchronization (`sync`)

The crate provides a simple `SpinLock<T>` suitable for early kernel code.

### 8.1 When to use it

- short critical sections
- early boot
- data structures shared across cores where sleeping is impossible or undesired

### 8.2 When not to use it

- long operations
- any code path that may block/sleep
- high-contention hot paths (you will need more advanced primitives)

### 8.3 Basic usage

```rust
use os_kernel_foundry::sync::SpinLock;

static COUNTER: SpinLock<u64> = SpinLock::new(0);

fn bump() {
    let mut guard = COUNTER.lock();
    *guard += 1;
}
```

---

## 9. Scheduling (`scheduler`)

Scheduling is modeled as *cooperative* and trait-based.

### 9.1 `SchedulableTask`

```rust
pub trait SchedulableTask {
    fn id(&self) -> u64;
    fn on_scheduled(&mut self);
}
```

### 9.2 `Scheduler<T>`

```rust
pub trait Scheduler<T: SchedulableTask> {
    fn add_task(&mut self, task: T);
    fn next_task(&mut self) -> Option<T>;
    fn len(&self) -> usize;
}
```

**Important**

- This crate does not perform context switches.
- It models only selection and bookkeeping.

In your kernel, `next_task()` might return a task control block handle, and your arch/platform layer will do the actual switch.

---

## 10. IPC (`ipc`)

IPC is modeled as message-based endpoints.

### 10.1 `Message`

```rust
pub trait Message {
    fn message_type(&self) -> &'static str;
}
```

### 10.2 `MessageEndpoint<M>`

```rust
pub trait MessageEndpoint<M: Message> {
    fn send(&mut self, msg: M) -> bool;
    fn recv(&mut self) -> Option<M>;
    fn len(&self) -> usize;
}
```

**Design intent**

- Keep the contract small.
- Allow deterministic in-memory channels in tests.

In a real kernel you might layer:

- blocking semantics
- back-pressure
- capabilities/permissions
- per-process mailboxing

---

## 11. Testing strategy

The crate itself demonstrates the intended usage style: most modules include `#[cfg(test)]` unit tests.

### 11.1 Host-side mocks

A recommended approach in your kernel project:

- build a `TestArch` that implements `Architecture` using in-memory mocks
- write boot stages as normal `BootStage<TestArch>`
- run the full boot sequence under `cargo test`

### 11.2 Determinism goals

For reliable CI, tests should be:

- deterministic
- free of timing dependencies
- independent from the host OS state

Use:

- mock timers
- in-memory registries
- pure functions where possible

---

## 12. `no_std` usage patterns

### 12.1 Avoiding allocation

- `DeviceRegistry` is allocation-free by design.
- Boot stages should avoid allocating unless you know an allocator exists.

### 12.2 Collections

In `no_std` code, you usually cannot use `Vec`, `BTreeMap`, etc. unless you bring in `alloc` and a global allocator.

Strategies:

- fixed-capacity arrays + manual indexing
- custom slab allocators
- `heapless`-style data structures (in your consumer crate)

### 12.3 Logging

This crate does not define logging.

In your kernel you typically implement:

- early serial logging (UART)
- later structured logging

Boot stages can use that logging to make failures diagnosable.

---

## 13. Common pitfalls

### 13.1 Trying to make this crate a bootable kernel

This crate is not a kernel binary. You need a separate binary crate for:

- entrypoint (`_start`)
- linker script
- boot protocol (UEFI, Multiboot2, custom)

### 13.2 Overloading `Architecture` with global kernel state

It is tempting to store everything inside `A`. Prefer clear ownership:

- architecture services in `A`
- kernel subsystems in a separate kernel state
- explicit passing of references where appropriate

### 13.3 Long-held spinlocks

Spinlocks must protect short critical sections. If you need blocking behavior, build proper primitives.

### 13.4 Unclear `unsafe` invariants

When you implement `PhysicalMemoryAllocator` / `VirtualMemoryManager`, document:

- frame size assumptions
- alignment requirements
- mapping constraints
- ownership rules

Even if the trait uses `usize`, your implementation should enforce stronger invariants.
