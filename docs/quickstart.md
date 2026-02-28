# Quickstart

This guide shows how to use **os_kernel_foundry** as the foundation of your own kernel project.

## Prerequisites

- Rust toolchain (stable)
- A separate kernel crate / workspace where you will implement your architecture back-end(s)

## 1) Add the dependency

### Via `path`

```toml
[dependencies]
os_kernel_foundry = { path = "../os_kernel_foundry" }
```

### Via crates.io

```toml
[dependencies]
os_kernel_foundry = "0.1"
```

## 2) Implement an `Architecture`

At minimum you provide three services:

- a monotonic `Timer`
- an `InterruptController`
- an `AddressTranslator`

You typically keep your low-level implementations in your own project (e.g. `arch/x86_64`, `arch/aarch64`, `platform/qemu-virt`).

## 3) Define a boot pipeline

Boot is modeled as an ordered list of stages. Each stage implements `boot::BootStage<A>`.

Stages receive a `boot::BootContext` that gives controlled access to the architecture.

Typical stages:

- timer setup
- interrupt controller configuration
- early memory init
- device discovery/registry setup
- scheduler init

## 4) Run the boot sequence

You can run boot via:

- `Kernel::boot(&stages)` (recommended)
- `boot::run_boot_sequence(&mut arch, &stages)`

## 5) Register and initialize devices

Device registration uses `device::DeviceRegistry`, a fixed-capacity registry that does **not** allocate.

The storage is provided by the caller as a slice of `Option<&mut dyn DeviceDriver>`.

## 6) Validate on host with `cargo test`

The crate is designed to make most logic testable on the host.

In your own kernel crate, you can:

- provide deterministic mocks for architecture services
- run boot stages and subsystem logic under unit tests

## Suggested project layout (consumer)

A common approach in your *kernel project*:

- `kernel-core/` (portable logic; mostly safe Rust; tests)
- `kernel-arch/` (per-arch back-ends)
- `kernel-platform/` (board/QEMU glue)
- `kernel-bin/` (entrypoint, linker script, bootloader integration)

**os_kernel_foundry** fits mainly into `kernel-core/` as the abstraction boundary.
