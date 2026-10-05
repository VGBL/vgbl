use crate::hal::Hal;

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

    pub fn poll(&mut self) {}
}
