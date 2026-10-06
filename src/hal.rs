mod clock;
mod execution_context;
mod flash;
mod watchdog;

pub use clock::Clock;
pub use execution_context::ExecutionContext;
pub use flash::{EraseError, Flash, FlashError, Region, Sector, Sectors, WriteError};
pub use watchdog::{NoWatchdog, Watchdog, WatchdogError};

/// The contract between the core bootloader and a chip-specific implementation
///
/// The implementation owns its set-up peripheral instances and lends them to the core
pub trait Hal {
    type Clock: Clock;
    type Flash: Flash;
    type Watchdog: Watchdog;

    /// Where the bootloader is currently executing from
    fn get_execution_context() -> ExecutionContext;

    fn get_clock(&mut self) -> &mut Self::Clock;

    fn get_flash(&mut self) -> &mut Self::Flash;

    fn get_watchdog(&mut self) -> &mut Self::Watchdog;
}