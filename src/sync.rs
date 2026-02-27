//! Lightweight synchronisation primitives.
//!
//! These primitives are intentionally minimal and designed to work in
//! `no_std` environments. They are not meant to be drop-in replacements
//! for the full `std::sync` toolbox, but rather building blocks for OS
//! kernels that need fine control over blocking behaviour.

use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// A simple spinlock suitable for use in early kernel code.
///
/// The lock does not implement poisoning and should not be held across
/// operations that may sleep. Its main purpose is to protect small, fast
/// critical sections where blocking the current core is acceptable.
pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    /// Creates a new spinlock protecting `value`.
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    /// Acquires the lock, spinning until it becomes available.
    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        while self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            spin_loop();
        }

        SpinLockGuard { lock: self }
    }
}

/// A guard returned by [`SpinLock::lock`].
///
/// While the guard is alive, the underlying data is considered locked.
pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<'a, T> Deref for SpinLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Safety: the guard guarantees unique access to the value.
        unsafe { &*self.lock.value.get() }
    }
}

impl<'a, T> DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: the guard guarantees unique access to the value.
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<'a, T> Drop for SpinLockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::SpinLock;

    #[test]
    fn spinlock_allows_mutation() {
        let lock = SpinLock::new(0usize);
        {
            let mut guard = lock.lock();
            *guard += 1;
        }

        let guard = lock.lock();
        assert_eq!(*guard, 1);
    }
}

