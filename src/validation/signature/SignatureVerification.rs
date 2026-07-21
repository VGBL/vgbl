//  Public-key signature verification
pub trait SignatureVerification {
    /// Public key material, as accepted by this algorithm
    type PublicKey;
 
    /// Verify `signature` over `message` under `public_key`
    fn verify(&self, public_key: &Self::PublicKey, message: &[u8], signature: &[u8]) -> bool;
}