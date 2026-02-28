#![cfg_attr(not(test), no_std)]

//! `os_kernel_foundry`
//!
//! A highly testable, modular foundation crate for building Rust-based
//! operating systems. The goal is to let you:
//! - Design your kernel architecture in safe, idiomatic Rust.
//! - Prototype and validate boot flows and subsystems entirely in unit tests.
//! - Swap real hardware/architecture back-ends without changing core logic.
//!
//! The crate is structured around a few key concepts:
//! - [`boot`] – declarative boot pipelines composed of strongly-typed stages.
//! - [`arch`] – traits that describe architecture-specific concerns.
//! - [`memory`] – pluggable memory management abstractions.
//! - [`device`] – a minimal but expressive device-driver model.
//! - [`kernel`] – a small orchestration layer tying everything together.
//! - [`sync`] – simple synchronization primitives suitable for `no_std`.
//! - [`scheduler`] – abstractions for cooperative task scheduling.
//! - [`ipc`] – minimal, message-based inter-process communication traits.
//!
//! The design philosophy is:
//! - Every public abstraction must be executable and testable on a regular
//!   host using `cargo test`.
//! - The same abstractions must remain usable in a `#![no_std]` context
//!   when you plug in your own low-level implementations.
//!
//! All public items are documented in English to ease publication on
//! `crates.io` and GitHub.

pub mod arch;
pub mod boot;
pub mod device;
pub mod ipc;
pub mod kernel;
pub mod memory;
pub mod scheduler;
pub mod sync;
