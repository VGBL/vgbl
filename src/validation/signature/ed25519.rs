use crate::validation::signature::Verifier;

/// Ed25519 algorithm contract
pub trait Ed25519: Verifier {}

pub fn software() -> SoftwareEd25519 {
    SoftwareEd25519
}

/*
    Software implementation
*/

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SoftwareEd25519;

impl SoftwareEd25519 {
    pub const fn new() -> Self {
        Self
    }
}

impl Verifier for SoftwareEd25519 {
    type PublicKey = crate::validation::signature::Ed25519PublicKey;

    fn verify(&self, _public_key: &Self::PublicKey, _message: &[u8], _signature: &[u8]) -> bool {
        todo!("software Ed25519 verify")
    }
}

impl Ed25519 for SoftwareEd25519 {}