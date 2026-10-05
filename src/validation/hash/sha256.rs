use crate::validation::hash::Hash;

/// SHA-256 algorithm contract
pub trait Sha256: Hash {
    const DIGEST_LEN: usize = 32;
}

pub fn software() -> SoftwareSha256 {
    SoftwareSha256
}

/*
    Software implementation
*/

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SoftwareSha256;

impl SoftwareSha256 {
    pub const fn new() -> Self {
        Self
    }
}

impl Hash for SoftwareSha256 {

    fn digest(&self, _data: &[u8], _out: &mut [u8]) {
        todo!("software SHA-256")
    }
}

impl Sha256 for SoftwareSha256 {}