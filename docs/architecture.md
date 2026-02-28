# Architecture & design principles

## Core philosophy

**os_kernel_foundry** is built around a single constraint: **most of your kernel should be testable with `cargo test`**.

To achieve this, the crate encourages:

- modeling hardware/arch concerns as **traits**
- keeping portable kernel logic **pure and deterministic**
- providing a host-side implementation (mock or simulation) for exhaustive testing

## `no_std` vs `std`

The crate is compiled as `no_std` in non-test builds:

- outside tests: `#![no_std]`
- in tests: `std` is available and used by test helpers and mock implementations

This allows unit tests to use standard collections (e.g. `BTreeMap`, `VecDeque`) while keeping the production API suitable for kernels.

## Boundaries and composition

The crate is organized into independent subsystems with minimal contracts:

- `arch`: minimal architecture services (timer, interrupts, address translation)
- `boot`: a strongly typed boot pipeline (`BootStage` + `BootContext`)
- `memory`: traits for physical and virtual memory management
- `device`: allocation-free driver model + registry
- `sync`: early-kernel friendly `SpinLock<T>`
- `scheduler`: cooperative scheduling traits
- `ipc`: message-based IPC traits
- `kernel`: orchestration layer that ties boot and devices together

The intent is that your kernel code depends only on these traits and types, not on ad-hoc APIs.

## Boot pipeline model

A boot sequence is expressed as an ordered list of `BootStage<A>` values.

Key properties:

- **explicit ordering**: stages run in a deterministic order
- **testability**: stages can be executed in host unit tests with mock `A: Architecture`
- **error handling**: failures stop the pipeline immediately and return a `BootError`

The state machine is represented by `BootState`:

- `NotStarted`
- `Running { current_stage }`
- `Completed { stages_run }`
- `Failed { failed_stage }`

## Device registry model

The device subsystem avoids heap allocation by design.

- You provide the storage backing slice.
- The registry stores `&mut dyn DeviceDriver` references.

This is suited for early boot, where a global allocator might not be available yet.

## Safety and `unsafe`

The crate keeps `unsafe` confined to **interfaces that must reflect hardware reality**, notably the memory traits:

- `PhysicalMemoryAllocator`
- `VirtualMemoryManager`

Your concrete implementations should document and uphold their invariants.

## What to implement in your own kernel

This crate intentionally does not implement:

- page tables
- frame allocators
- interrupt descriptor tables / vector tables
- CPU startup/bring-up
- context switching

Instead it provides the boundary contracts that let you implement those pieces in a structured, testable way.
