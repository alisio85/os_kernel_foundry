# Documentation

Welcome to the **os_kernel_foundry** documentation.

This crate is a modular, test-driven foundation for building Rust-based operating systems. The central idea is to model architecture- and platform-specific concerns as **safe Rust traits**, so most of your kernel logic can be executed and validated with **deterministic unit tests** on a regular host.

## Contents

- [Quickstart](./quickstart.md)
- [Architecture & design principles](./architecture.md)
- [Module reference](./modules.md)
- [Testing & CI](./testing-ci.md)
- [Publishing (crates.io) & releases](./publishing.md)
- [Contributing](./contributing.md)

## What this crate is (and is not)

- **This crate is:** a set of portable abstractions + a small orchestration layer (`Kernel`) that helps you compose a boot pipeline and subsystem boundaries.
- **This crate is not:** a complete, runnable kernel, bootloader, HAL, or board support package. There are no linker scripts, CPU entrypoints, or MMU implementations included.

## Public API entrypoints (high level)

- `kernel::Kernel<A>`
  - `Kernel::new(arch)`
  - `Kernel::boot(stages)`
  - `Kernel::init_devices(registry)`
- `boot::run_boot_sequence(arch, stages)`

## Feature flags

- `std`: enables `std` support for host-side experimentation and unit tests. The crate is `no_std` outside tests.
