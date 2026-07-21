//  A checksum is keyless and produces a check value primarily for corruption detection
pub trait Checksum {
    /// Width of the check value in bytes
    const OUTPUT_LEN: usize;
 
    /// Compute the check value over `data`, writing it into `out`
    fn compute(&self, data: &[u8], out: &mut [u8]);
 
    /// Compute and compare against an expected value
    fn verify(&self, data: &[u8], expected: &[u8]) -> bool;
}