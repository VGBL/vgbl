/// A watchdog that resets the chip unless it's fed in time
///
/// Construction, including the timeout, is left to each implementation
pub trait Watchdog {
    /// Restarts the timeout. Does nothing while disabled
    fn feed(&mut self);

    /// Starts the watchdog. Does nothing if it's already running
    fn enable(&mut self) -> Result<(), WatchdogError>;

    /// Stops the watchdog. Does nothing if it isn't running
    fn disable(&mut self) -> Result<(), WatchdogError>;
}

/// For chips or builds without a watchdog. `enable` fails so the core knows nothing is protecting it
pub struct NoWatchdog;

impl Watchdog for NoWatchdog {
    fn feed(&mut self) {}

    fn enable(&mut self) -> Result<(), WatchdogError> {
        Err(WatchdogError::Unsupported)
    }

    fn disable(&mut self) -> Result<(), WatchdogError> {
        Ok(())
    }
}

/// Why a watchdog operation failed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatchdogError {
    /// The hardware can't do this, e.g. a watchdog that only a reset can stop
    Unsupported,
}
