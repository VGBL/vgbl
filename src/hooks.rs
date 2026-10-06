use crate::Bootloader;
use crate::hal::Hal;

/// Points where an implementation extends the core bootloader
///
/// Every hook defaults to doing nothing, so an implementation only overrides the ones it needs. Hooks are passed into
/// the bootloader's calls rather than owned by it, so each one can be handed the whole bootloader
pub trait Hooks<H: Hal> {
    /// Once, after the chip is set up and before the first `poll`
    fn init(&mut self, _bootloader: &mut Bootloader<H>) {}

    /// Every `poll`, after the core has run
    fn poll(&mut self, _bootloader: &mut Bootloader<H>) {}

    /// Right before jumping to a payload, to undo anything set up for the bootloader (de-asserting GPIO, etc)
    fn exit(&mut self, _bootloader: &mut Bootloader<H>) {}
}
