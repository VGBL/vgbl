use crate::validation::signature::SignatureVerification;

/// ECDSA P-256 algorithm contract
pub trait EcdsaP256: Verifier {}

pub fn software() -> SoftwareEcdsaP256 {
    SoftwareEcdsaP256
}

/*
    Software implementation
*/

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SoftwareEcdsaP256;

impl SoftwareEcdsaP256 {
    pub const fn new() -> Self {
        Self
    }
}

impl Verifier for SoftwareEcdsaP256 {
    type PublicKey = crate::validation::signature::EcdsaP256PublicKey;

    fn verify(&self, _public_key: &Self::PublicKey, _message: &[u8], _signature: &[u8]) -> bool {
        todo!("software P-256 verify")
    }
}

impl EcdsaP256 for SoftwareEcdsaP256 {}