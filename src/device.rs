//! Device driver abstractions and registry.
//!
//! The core idea is to allow kernels to express their device topology in a
//! way that is:
//! - independent of any particular bus (PCI, MMIO, platform devices),
//! - testable on a regular host, and
//! - free from dynamic allocation requirements.

/// Errors that can be returned by device drivers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceError {
    /// A generic initialisation failure.
    InitFailed,
}

/// A minimal device driver interface.
pub trait DeviceDriver {
    /// Returns a short, human-readable name, used mainly for logging.
    fn name(&self) -> &'static str;

    /// Performs device initialisation.
    fn init(&mut self) -> Result<(), DeviceError>;
}

/// A fixed-capacity device registry.
///
/// The registry does not allocate; instead, it stores devices in a
/// caller-provided backing slice. This works both in `no_std` kernels and
/// in host-side tests.
pub struct DeviceRegistry<'a> {
    devices: &'a mut [Option<&'a mut dyn DeviceDriver>],
    count: usize,
}

impl<'a> DeviceRegistry<'a> {
    /// Creates a new registry backed by the given slice.
    pub fn new(storage: &'a mut [Option<&'a mut dyn DeviceDriver>]) -> Self {
        Self {
            devices: storage,
            count: 0,
        }
    }

    /// Returns the number of registered devices.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns `true` if no devices are currently registered.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Attempts to register a new device.
    ///
    /// Returns `false` if the registry is full.
    pub fn register(&mut self, device: &'a mut dyn DeviceDriver) -> bool {
        if self.count >= self.devices.len() {
            return false;
        }

        self.devices[self.count] = Some(device);
        self.count += 1;
        true
    }

    /// Initialises all registered devices, stopping at the first failure.
    pub fn init_all(&mut self) -> Result<(), DeviceError> {
        for index in 0..self.count {
            if let Some(device) = &mut self.devices[index] {
                device.init()?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{DeviceDriver, DeviceError, DeviceRegistry};

    struct GoodDevice {
        initialised: bool,
    }

    impl GoodDevice {
        fn new() -> Self {
            Self { initialised: false }
        }
    }

    impl DeviceDriver for GoodDevice {
        fn name(&self) -> &'static str {
            "good"
        }

        fn init(&mut self) -> Result<(), DeviceError> {
            self.initialised = true;
            Ok(())
        }
    }

    struct BadDevice;

    impl DeviceDriver for BadDevice {
        fn name(&self) -> &'static str {
            "bad"
        }

        fn init(&mut self) -> Result<(), DeviceError> {
            Err(DeviceError::InitFailed)
        }
    }

    #[test]
    fn register_and_init_devices() {
        let mut good = GoodDevice::new();
        let mut bad = BadDevice;
        let mut storage: [Option<&mut dyn DeviceDriver>; 2] = [None, None];

        let mut registry = DeviceRegistry::new(&mut storage);
        assert!(registry.register(&mut good));
        assert!(registry.register(&mut bad));
        assert_eq!(registry.len(), 2);

        // Initialisation should stop at the failing device.
        let result = registry.init_all();
        assert!(result.is_err());
    }
}
