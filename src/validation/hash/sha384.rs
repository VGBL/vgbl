use crate::validation::hash::Hash;

/// SHA-384 algorithm contract
pub trait Sha384: Hash {
    const DIGEST_LEN: usize = 48;
}

pub fn software() -> SoftwareSha384 {
    SoftwareSha384
}

/*
    Software implementation
*/

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SoftwareSha384;

impl SoftwareSha384 {
    pub const fn new() -> Self {
        Self
    }
}

impl Hash for SoftwareSha384 {
    fn digest(&self, _data: &[u8], _out: &mut [u8]) {
        todo!("software SHA-384")
    }
}

impl Sha384 for SoftwareSha384 {}