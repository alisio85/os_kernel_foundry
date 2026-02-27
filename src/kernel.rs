//! High-level kernel orchestration.
//!
//! This module ties together the abstractions from [`crate::arch`],
//! [`crate::boot`], [`crate::device`] and [`crate::memory`]. The goal is
//! not to prescribe a single kernel architecture, but to provide a small,
//! testable skeleton that can be extended for real-world systems.

use crate::arch::Architecture;
use crate::boot::{run_boot_sequence, BootError, BootStage, BootState};
use crate::device::DeviceRegistry;

/// Represents the high-level kernel, parameterised over an architecture.
pub struct Kernel<A: Architecture> {
    arch: A,
}

impl<A: Architecture> Kernel<A> {
    /// Creates a new kernel instance from the given architecture.
    pub fn new(arch: A) -> Self {
        Self { arch }
    }

    /// Returns a shared reference to the underlying architecture.
    pub fn arch(&self) -> &A {
        &self.arch
    }

    /// Returns a mutable reference to the underlying architecture.
    pub fn arch_mut(&mut self) -> &mut A {
        &mut self.arch
    }

    /// Runs the boot sequence using the provided stages.
    pub fn boot(
        &mut self,
        stages: &[&dyn BootStage<A>],
    ) -> Result<BootState, BootError> {
        run_boot_sequence(&mut self.arch, stages)
    }

    /// Initialises all devices in the provided registry.
    pub fn init_devices<'a>(
        &mut self,
        registry: &mut DeviceRegistry<'a>,
    ) -> Result<(), crate::device::DeviceError> {
        let _ = &mut self.arch;
        registry.init_all()
    }
}

#[cfg(test)]
mod tests {
    use super::Kernel;
    use crate::arch::{
        Architecture, MockAddressTranslator, MockInterruptController, MockTimer, Timer,
    };
    use crate::boot::{BootContext, BootError, BootStage, BootState};
    use crate::device::{DeviceDriver, DeviceError, DeviceRegistry};

    #[derive(Debug)]
    struct TestArch {
        timer: MockTimer,
        ic: MockInterruptController,
        translator: MockAddressTranslator,
    }

    impl TestArch {
        fn new() -> Self {
            Self {
                timer: MockTimer::new(),
                ic: MockInterruptController::default(),
                translator: MockAddressTranslator::new(),
            }
        }
    }

    impl Architecture for TestArch {
        type Timer = MockTimer;
        type InterruptController = MockInterruptController;
        type AddressTranslator = MockAddressTranslator;

        fn timer(&self) -> &Self::Timer {
            &self.timer
        }

        fn interrupt_controller(&mut self) -> &mut Self::InterruptController {
            &mut self.ic
        }

        fn address_translator(&self) -> &Self::AddressTranslator {
            &self.translator
        }
    }

    struct QuickBoot;

    impl BootStage<TestArch> for QuickBoot {
        fn name(&self) -> &'static str {
            "quick-boot"
        }

        fn run(&self, ctx: &mut BootContext<'_, TestArch>) -> Result<(), BootError> {
            let _ = ctx.arch().timer().now();
            Ok(())
        }
    }

    struct DummyDevice;

    impl DeviceDriver for DummyDevice {
        fn name(&self) -> &'static str {
            "dummy"
        }

        fn init(&mut self) -> Result<(), DeviceError> {
            Ok(())
        }
    }

    #[test]
    fn kernel_boot_and_device_init() {
        let arch = TestArch::new();
        let mut kernel = Kernel::new(arch);

        let boot_stages: [&dyn BootStage<TestArch>; 1] = [&QuickBoot];
        let state = kernel.boot(&boot_stages).expect("boot should succeed");
        assert!(matches!(state, BootState::Completed { .. }));

        let mut device = DummyDevice;
        let mut storage: [Option<&mut dyn DeviceDriver>; 1] = [None];
        let mut registry = DeviceRegistry::new(&mut storage);
        assert!(registry.register(&mut device));

        kernel.init_devices(&mut registry).expect("devices should init");
    }
}

