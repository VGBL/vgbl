/// Where the bootloader is executing from, relative to the flash it manages
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionContext {
    /// Code or vectors are fetched from memory that stalls during flash operations, so they must block
    Flash,

    /// Code and vectors are in RAM, so execution continues while a flash operation runs
    Ram,
}
