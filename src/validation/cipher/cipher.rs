//  Symmetric cipher
pub trait Cipher {
    /// Key material.
    type Key;
 
    /// Decrypt `input` into `output` under `key`. Returns the number of bytes written, or `None` on failure.
    fn decrypt(&self, key: &Self::Key, input: &[u8], output: &mut [u8]) -> Option<usize>;
}