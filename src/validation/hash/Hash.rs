/// A cryptographic hash is keyless and produces a fixed-length integrity value
pub trait Hash {
    /// Digest length in bytes
    const DIGEST_LEN: usize;
 
    /// Compute the digest of `data` into `out` (must be at least `DIGEST_LEN`)
    fn digest(&self, data: &[u8], out: &mut [u8]);
}