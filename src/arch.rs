//! Architecture abstractions.
//!
//! This module defines traits that describe the minimal contract between
//! portable kernel code and architecture-specific back-ends. The idea is
//! that your high-level kernel logic never touches raw CPU instructions
//! directly; instead, it operates against these traits, which can be
//! implemented both by:
//! - real, low-level hardware back-ends, and
//! - high-level host-side mocks used in unit tests.

/// Describes a monotonic timer source used by the kernel.
///
/// The timer is intentionally minimal: it only provides a way to read a
/// monotonically increasing tick counter. Conversion to human-readable
/// time units is left to higher layers.
pub trait Timer {
    /// The underlying tick type exposed by the timer implementation.
    type Tick: Copy + Ord;

    /// Returns the current monotonic tick value.
    fn now(&self) -> Self::Tick;
}

/// Describes the basic behaviour of an interrupt controller.
///
/// Real interrupt controllers are complex, but most kernels only need a
/// small, well-defined subset of the behaviour at any given time. This
/// trait focuses on that subset.
pub trait InterruptController {
    /// Enables the interrupt line associated with `id`.
    fn enable(&mut self, id: u32);

    /// Disables the interrupt line associated with `id`.
    fn disable(&mut self, id: u32);

    /// Acknowledge the interrupt associated with `id`, signalling that
    /// the kernel has completed its handler.
    fn acknowledge(&mut self, id: u32);
}

/// Minimal interface for address translation.
///
/// Real-world address translation is deeply architecture-specific, but
/// from the kernel's point of view, the essential operation is "turn a
/// virtual address into an optional physical address".
pub trait AddressTranslator {
    /// Performs a best-effort translation of `virtual_address`.
    ///
    /// Implementations may choose to return `None` if the mapping does
    /// not exist or cannot be resolved in the current context.
    fn translate(&self, virtual_address: usize) -> Option<usize>;
}

/// Aggregates architecture services required by the higher-level kernel.
///
/// This trait is designed so that your kernel entry point can be generic
/// over `A: Architecture`, which in turn allows you to:
/// - run the same kernel logic on multiple hardware back-ends, and
/// - use purely in-memory mocks for exhaustive testing.
pub trait Architecture {
    /// The timer implementation used by this architecture.
    type Timer: Timer;

    /// The interrupt controller implementation used by this architecture.
    type InterruptController: InterruptController;

    /// The address translator implementation used by this architecture.
    type AddressTranslator: AddressTranslator;

    /// Returns a shared reference to the architecture timer.
    fn timer(&self) -> &Self::Timer;

    /// Returns a mutable reference to the interrupt controller.
    fn interrupt_controller(&mut self) -> &mut Self::InterruptController;

    /// Returns a shared reference to the address translator.
    fn address_translator(&self) -> &Self::AddressTranslator;
}

/// A purely in-memory timer implementation intended for tests.
///
/// It can be advanced manually, which makes it ideal for deterministic
/// unit tests that need to reason about time without relying on the host
/// operating system.
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MockTimer {
    tick: u64,
}

#[cfg(test)]
impl MockTimer {
    /// Creates a new mock timer starting at tick `0`.
    pub fn new() -> Self {
        Self { tick: 0 }
    }

    /// Advances the timer by `delta` ticks and returns the new value.
    pub fn advance(&mut self, delta: u64) -> u64 {
        self.tick = self.tick.saturating_add(delta);
        self.tick
    }
}

#[cfg(test)]
impl Timer for MockTimer {
    type Tick = u64;

    fn now(&self) -> Self::Tick {
        self.tick
    }
}

/// A minimal in-memory interrupt controller used for tests.
#[cfg(test)]
#[derive(Debug, Default)]
pub struct MockInterruptController {
    last_enabled: Option<u32>,
    last_disabled: Option<u32>,
    last_acknowledged: Option<u32>,
}

#[cfg(test)]
impl InterruptController for MockInterruptController {
    fn enable(&mut self, id: u32) {
        self.last_enabled = Some(id);
    }

    fn disable(&mut self, id: u32) {
        self.last_disabled = Some(id);
    }

    fn acknowledge(&mut self, id: u32) {
        self.last_acknowledged = Some(id);
    }
}

/// A trivial translator that uses an in-memory map.
#[cfg(test)]
#[derive(Debug, Default)]
pub struct MockAddressTranslator {
    // In a real implementation this would likely be a more sophisticated
    // structure. For tests we rely on `std::collections::BTreeMap`, which
    // is available because tests are executed with `std` support enabled.
    mappings: std::collections::BTreeMap<usize, usize>,
}

#[cfg(test)]
impl MockAddressTranslator {
    /// Creates an empty translator with no mappings.
    pub fn new() -> Self {
        Self {
            mappings: std::collections::BTreeMap::new(),
        }
    }

    /// Inserts a new mapping from `virtual_address` to `physical_address`.
    pub fn insert(&mut self, virtual_address: usize, physical_address: usize) {
        let _ = self.mappings.insert(virtual_address, physical_address);
    }
}

#[cfg(test)]
impl AddressTranslator for MockAddressTranslator {
    fn translate(&self, virtual_address: usize) -> Option<usize> {
        self.mappings.get(&virtual_address).copied()
    }
}
