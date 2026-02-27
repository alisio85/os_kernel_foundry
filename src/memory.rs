//! Memory management abstractions.
//!
//! This module deliberately avoids prescribing a specific paging or
//! segmentation model. Instead, it focuses on portable traits that your
//! architecture- and platform-specific code can implement.

/// Describes an allocator for fixed-size physical frames.
///
/// The semantics are intentionally minimal: the allocator hands out opaque
/// frame identifiers and expects the caller to know how to interpret them.
pub trait PhysicalMemoryAllocator {
    /// Allocates a single frame and returns its identifier.
    ///
    /// Safety: kernel code must ensure that each allocated frame is either
    /// mapped exactly once or otherwise tracked to avoid aliasing.
    unsafe fn allocate_frame(&mut self) -> Option<usize>;

    /// Deallocates a previously allocated frame.
    ///
    /// Safety: callers must guarantee that `frame` was previously returned
    /// by [`PhysicalMemoryAllocator::allocate_frame`] and is no longer in
    /// active use.
    unsafe fn deallocate_frame(&mut self, frame: usize);
}

/// Describes a minimal virtual memory manager.
pub trait VirtualMemoryManager {
    /// Establishes a mapping from `virtual_address` to `physical_address`.
    ///
    /// Implementations may impose alignment or size constraints which
    /// should be documented alongside the concrete type.
    unsafe fn map(&mut self, virtual_address: usize, physical_address: usize) -> Result<(), ()>;

    /// Removes the mapping for `virtual_address`, if any.
    unsafe fn unmap(&mut self, virtual_address: usize) -> Result<(), ()>;
}

#[cfg(test)]
mod tests {
    use super::{PhysicalMemoryAllocator, VirtualMemoryManager};
    use std::collections::BTreeMap;

    struct TestAllocator {
        next: usize,
    }

    impl TestAllocator {
        fn new() -> Self {
            Self { next: 1 }
        }
    }

    impl PhysicalMemoryAllocator for TestAllocator {
        unsafe fn allocate_frame(&mut self) -> Option<usize> {
            let frame = self.next;
            self.next = self.next.saturating_add(1);
            Some(frame)
        }

        unsafe fn deallocate_frame(&mut self, _frame: usize) {
            // In a test allocator we intentionally do nothing. Real
            // implementations would return the frame to a free list.
        }
    }

    struct TestVirtualMemory {
        mappings: BTreeMap<usize, usize>,
    }

    impl TestVirtualMemory {
        fn new() -> Self {
            Self {
                mappings: BTreeMap::new(),
            }
        }
    }

    impl VirtualMemoryManager for TestVirtualMemory {
        unsafe fn map(&mut self, virtual_address: usize, physical_address: usize) -> Result<(), ()> {
            self.mappings.insert(virtual_address, physical_address);
            Ok(())
        }

        unsafe fn unmap(&mut self, virtual_address: usize) -> Result<(), ()> {
            self.mappings.remove(&virtual_address);
            Ok(())
        }
    }

    #[test]
    fn allocate_and_map_frame() {
        let mut allocator = TestAllocator::new();
        let mut vm = TestVirtualMemory::new();

        let frame = unsafe { allocator.allocate_frame() }.expect("frame");
        unsafe { vm.map(0x1000, frame).expect("map") };
    }
}

