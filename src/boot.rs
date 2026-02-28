//! Boot pipeline abstractions.
//!
//! The goal of this module is to make the boot process of your operating
//! system:
//! - explicitly modelled in types,
//! - easy to test on a regular host, and
//! - easy to port across architectures.
//!
//! A boot sequence is expressed as an ordered list of stages. Each stage
//! operates on a shared [`BootContext`] and may return an error to abort
//! the sequence.

use crate::arch::Architecture;

/// Error type returned by boot stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootError {
    /// A fatal error with a short, static description.
    Fatal(&'static str),
}

/// Represents the progress of the boot sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootState {
    /// Boot has not yet begun.
    NotStarted,
    /// Boot is currently executing stage at the given index.
    Running { current_stage: usize },
    /// Boot completed successfully after the given number of stages.
    Completed { stages_run: usize },
    /// Boot failed at the given stage index.
    Failed { failed_stage: usize },
}

/// Shared context passed to each boot stage.
///
/// The context provides controlled access to architecture services while
/// tracking global boot state. It is intentionally conservative: stages
/// cannot replace the architecture instance and can only mutate it through
/// the returned mutable reference.
pub struct BootContext<'a, A: Architecture> {
    arch: &'a mut A,
    state: BootState,
}

impl<'a, A: Architecture> BootContext<'a, A> {
    /// Creates a new context wrapping the provided architecture instance.
    pub fn new(arch: &'a mut A) -> Self {
        Self {
            arch,
            state: BootState::NotStarted,
        }
    }

    /// Returns a mutable reference to the underlying architecture.
    ///
    /// Boot stages use this method to interact with timers, interrupt
    /// controllers and address translation without depending on concrete
    /// hardware types.
    pub fn arch(&mut self) -> &mut A {
        self.arch
    }

    /// Returns the current boot state.
    pub fn state(&self) -> BootState {
        self.state
    }

    fn set_state(&mut self, state: BootState) {
        self.state = state;
    }
}

/// A single stage in a boot pipeline.
///
/// Stages should be small and focused on one responsibility: for example
/// "initialise memory management" or "configure the timer". This makes
/// them both easier to reason about and trivial to unit test.
pub trait BootStage<A: Architecture> {
    /// Returns a short, human-readable name for the stage.
    fn name(&self) -> &'static str;

    /// Executes the stage using the provided [`BootContext`].
    fn run(&self, ctx: &mut BootContext<'_, A>) -> Result<(), BootError>;
}

/// Executes all boot stages in order.
///
/// If any stage returns an error, the sequence halts immediately and the
/// error is returned to the caller. On success, the final [`BootState`]
/// is guaranteed to be [`BootState::Completed`].
pub fn run_boot_sequence<A: Architecture>(
    arch: &mut A,
    stages: &[&dyn BootStage<A>],
) -> Result<BootState, BootError> {
    let mut ctx = BootContext::new(arch);

    for (index, stage) in stages.iter().enumerate() {
        ctx.set_state(BootState::Running {
            current_stage: index,
        });

        stage.run(&mut ctx).map_err(|err| {
            ctx.set_state(BootState::Failed {
                failed_stage: index,
            });
            err
        })?;
    }

    ctx.set_state(BootState::Completed {
        stages_run: stages.len(),
    });

    Ok(ctx.state())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::{
        Architecture, MockAddressTranslator, MockInterruptController, MockTimer, Timer,
    };

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

    struct TickStage;

    impl<A> BootStage<A> for TickStage
    where
        A: Architecture,
        A::Timer: Timer<Tick = u64>,
    {
        fn name(&self) -> &'static str {
            "tick"
        }

        fn run(&self, ctx: &mut BootContext<'_, A>) -> Result<(), BootError> {
            let current = ctx.arch().timer().now();
            if current > u64::MAX - 1 {
                return Err(BootError::Fatal("tick overflow"));
            }
            Ok(())
        }
    }

    struct FailingStage;

    impl<A: Architecture> BootStage<A> for FailingStage {
        fn name(&self) -> &'static str {
            "failing"
        }

        fn run(&self, _ctx: &mut BootContext<'_, A>) -> Result<(), BootError> {
            Err(BootError::Fatal("expected failure"))
        }
    }

    #[test]
    fn boot_sequence_succeeds() {
        let mut arch = TestArch::new();
        let stages: [&dyn BootStage<TestArch>; 1] = [&TickStage];

        let state = run_boot_sequence(&mut arch, &stages).expect("boot should succeed");
        assert_eq!(
            state,
            BootState::Completed {
                stages_run: stages.len()
            }
        );
    }

    #[test]
    fn boot_sequence_stops_on_failure() {
        let mut arch = TestArch::new();
        let stages: [&dyn BootStage<TestArch>; 2] = [&TickStage, &FailingStage];

        let result = run_boot_sequence(&mut arch, &stages);
        assert!(result.is_err());
    }
}
