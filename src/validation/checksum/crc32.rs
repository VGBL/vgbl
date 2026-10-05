use crate::validation::checksum::Checksum;

/// CRC-32 algorithm contract
pub trait Crc32: Checksum {
    const OUTPUT_LEN: usize = 4;   // algorithm-level default
}

pub fn software() -> SoftwareCrc32 {
    SoftwareCrc32
}

/*
    Software implementation
*/

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SoftwareCrc32;

impl SoftwareCrc32 {
    pub const fn new() -> Self {
        Self
    }
}

impl Checksum for SoftwareCrc32 {
    fn compute(&self, _data: &[u8], _out: &mut [u8]) {
        todo!("software CRC-32 computation")
    }

    fn verify(&self, _data: &[u8], _expected: &[u8]) -> bool {
        todo!("software CRC-32 verify")
    }
}

impl Crc32 for SoftwareCrc32 {}