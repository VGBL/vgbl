use crate::hal::Hal;
use crate::hooks::Hooks;

/// The core bootloader
///
/// Owns everything the application sets up and drives it from `poll`, which the application calls from its main
/// loop. The functionality itself lives in sub-modules
pub struct Bootloader<H: Hal> {
    hal: H,
}

impl<H: Hal> Bootloader<H> {
    pub fn new(hal: H) -> Self {
        Self { hal }
    }

    /// The implementation, for anything chip-specific outside the HAL traits
    pub fn hal(&mut self) -> &mut H {
        &mut self.hal
    }

    /// Call once before the first `poll`
    pub fn init(&mut self, hooks: &mut impl Hooks<H>) {
        hooks.init(self);
    }

    /// Call from the main loop
    pub fn poll(&mut self, hooks: &mut impl Hooks<H>) {
        hooks.poll(self);
    }
}
