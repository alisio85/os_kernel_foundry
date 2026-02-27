//! Cooperative scheduling abstractions.
//!
//! This module provides minimal traits and utilities for building a
//! scheduler on top of `os_kernel_foundry`. The focus is on testability and
//! independence from any particular threading or context-switching model.

/// Represents a schedulable task.
///
/// In a real kernel this might wrap a thread control block, a process,
/// or any other runnable entity. Here we keep the contract deliberately
/// small and high-level.
pub trait SchedulableTask {
    /// Returns a unique, stable identifier for the task.
    fn id(&self) -> u64;

    /// Called by the scheduler when the task is selected to run.
    fn on_scheduled(&mut self);
}

/// Describes a cooperative scheduler.
///
/// The scheduler is responsible for deciding which task should run next,
/// but it does not perform context switches itself. That responsibility
/// remains with the surrounding kernel code.
pub trait Scheduler<T: SchedulableTask> {
    /// Adds a new task to the run queue.
    fn add_task(&mut self, task: T);

    /// Selects the next task to run, if any.
    fn next_task(&mut self) -> Option<T>;

    /// Returns the number of tasks currently known to the scheduler.
    fn len(&self) -> usize;

    /// Returns `true` if the scheduler has no tasks available.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::{SchedulableTask, Scheduler};

    #[derive(Debug, Clone)]
    struct TestTask {
        id: u64,
        scheduled_count: u32,
    }

    impl TestTask {
        fn new(id: u64) -> Self {
            Self {
                id,
                scheduled_count: 0,
            }
        }
    }

    impl SchedulableTask for TestTask {
        fn id(&self) -> u64 {
            self.id
        }

        fn on_scheduled(&mut self) {
            self.scheduled_count = self.scheduled_count.saturating_add(1);
        }
    }

    /// A very small round-robin scheduler used only for tests and examples.
    struct RoundRobinScheduler {
        queue: std::collections::VecDeque<TestTask>,
    }

    impl RoundRobinScheduler {
        fn new() -> Self {
            Self {
                queue: std::collections::VecDeque::new(),
            }
        }
    }

    impl Scheduler<TestTask> for RoundRobinScheduler {
        fn add_task(&mut self, task: TestTask) {
            self.queue.push_back(task);
        }

        fn next_task(&mut self) -> Option<TestTask> {
            self.queue.pop_front()
        }

        fn len(&self) -> usize {
            self.queue.len()
        }
    }

    #[test]
    fn round_robin_schedules_all_tasks() {
        let mut scheduler = RoundRobinScheduler::new();

        scheduler.add_task(TestTask::new(1));
        scheduler.add_task(TestTask::new(2));
        scheduler.add_task(TestTask::new(3));

        let mut seen_ids = std::collections::BTreeSet::new();

        while let Some(mut task) = scheduler.next_task() {
            task.on_scheduled();
            assert_eq!(task.scheduled_count, 1);
            seen_ids.insert(task.id());
        }

        assert_eq!(seen_ids.len(), 3);
        assert!(seen_ids.contains(&1));
        assert!(seen_ids.contains(&2));
        assert!(seen_ids.contains(&3));
    }
}

