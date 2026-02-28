# Module reference

This page summarizes each public module and its key items.

## `arch`

Defines architecture traits:

- `Timer` (monotonic tick source)
- `InterruptController` (enable/disable/acknowledge)
- `AddressTranslator` (virtual → physical translation)
- `Architecture` (aggregates the above)

In `#[cfg(test)]` builds the crate provides in-memory mocks used by unit tests.

## `boot`

Boot pipeline model:

- `BootStage<A>`: one initialization stage
- `BootContext<'_, A>`: context passed to each stage
- `BootState`: progress of the pipeline
- `BootError`: error returned by stages
- `run_boot_sequence`: executes stages sequentially

## `kernel`

High-level orchestration:

- `Kernel<A>`
  - `new(arch)`
  - `boot(stages)`
  - `init_devices(registry)`

`Kernel` is intentionally small: it exists to standardize composition rather than enforce a full architecture.

## `device`

Device driver model:

- `DeviceDriver`: minimal driver trait (`name`, `init`)
- `DeviceRegistry`: fixed-capacity registry backed by caller-provided storage
- `DeviceError`: error enum for driver init

## `memory`

Portable memory traits:

- `PhysicalMemoryAllocator`
- `VirtualMemoryManager`

The APIs are `unsafe` to reflect kernel invariants (aliasing, mapping correctness, etc.).

## `sync`

Synchronization primitives suitable for early boot:

- `SpinLock<T>`
- `SpinLockGuard<'_, T>`

## `scheduler`

Cooperative scheduling contracts:

- `SchedulableTask`
- `Scheduler<T>`

## `ipc`

Message-based IPC contracts:

- `Message`
- `MessageEndpoint<M>`
