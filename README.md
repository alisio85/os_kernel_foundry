# os_kernel_foundry

`os_kernel_foundry` is a modular, heavily test-driven foundation crate
for building Rust-based operating systems, created in 2026.

The core idea is simple:

- Model your kernel architecture as pure, safe Rust traits.
- Implement those traits twice:
  - once for real hardware, and
  - once for fast, deterministic host-side mocks.
- Write the majority of your OS logic and boot flow so that it is completely
  testable with `cargo test`, long before you run anything on bare metal.

## Design goals

- **No_std friendly**: the library compiles without `std` for kernel usage,
  while tests run with `std` enabled for convenience.
- **Strict boundaries**: high-level kernel code talks only to traits
  (`Architecture`, `BootStage`, `DeviceDriver`, `PhysicalMemoryAllocator`,
  `VirtualMemoryManager`, `SpinLock<T>`), never to ad-hoc, unstructured APIs.
- **Deterministic tests**: all provided mock implementations are designed to
  be deterministic and side-effect free, making them ideal for CI.

## Module overview

- `arch`: core architecture traits (`Timer`, `InterruptController`,
  `AddressTranslator`, `Architecture`) and host-side mocks.
- `boot`: strongly-typed boot pipeline built from `BootStage` values and
  executed via `run_boot_sequence`.
- `memory`: portable traits for physical and virtual memory management.
- `device`: allocation-free `DeviceRegistry` and `DeviceDriver` trait.
- `sync`: a minimal `SpinLock<T>` for early-kernel synchronisation.
- `kernel`: a small orchestration type that ties the pieces together.
- `scheduler`: abstractions for cooperative, testable task scheduling.
- `ipc`: minimal message-based IPC traits and test-friendly channels.

## Basic usage

1. Define your own architecture type that implements `Architecture`.
2. Implement one or more `BootStage<A>` stages for initialisation.
3. (Optional) Define your own device drivers and registry.
4. Wrap everything in `Kernel<A>` and call `Kernel::boot`.

See the inline documentation in `src/` for full, commented examples and the
unit tests for realistic compositions.

## Getting started

1. Add `os_kernel_foundry` as a dependency (via path or crates.io).
2. Create an architecture type that implements `Architecture` (timer,
   interrupt controller, address translator).
3. Define a small set of `BootStage<A>` implementations (memory init,
   device init, scheduler init, etc.).
4. Wrap your architecture in `Kernel<A>` and call `Kernel::boot`.
5. Use `DeviceRegistry` and `DeviceDriver` to organise early devices.
6. Optionally, build your own scheduler on top of the `scheduler` module
   and layer IPC on top of the `ipc` traits.

    ```markdown
     ![CI](https://github.com/<tuo-utente>/os_kernel_foundry/actions/workflows/ci.yml/badge.svg)
     ```

## License

This project is licensed under the MIT License. See `LICENSE` for details.

