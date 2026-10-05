use core::ops::Range;

/// Erasing and writing internal flash
///
/// Every operation follows the same pattern: `start_*` begins it, then `poll` drives it until it completes. Only
/// one operation runs at a time. How the work runs depends on the execution context:
/// * RAM: `start_*` returns as soon as the hardware is started, and `poll` returns
///   [`WouldBlock`](nb::Error::WouldBlock) until it finishes
/// * Flash: the CPU can't run while flash is busy, so `start_*` does the work before returning and the next `poll`
///   reports the result
///
/// Either way, a single call blocks for at most one sector erase, so the caller gets control back between sectors
pub trait Flash {
    /// Largest `data` accepted by `start_write`
    const MAX_WRITE: usize;

    /// `start_write` address and length must both be multiples of this
    const WRITE_ALIGN: usize;

    /// Every erasable sector, in ascending address order with no gaps
    fn get_sectors(&self) -> &[Sector];

    /// Starts erasing one sector. `sector` is an index into `get_sectors`
    fn start_erase_sector(&mut self, sector: usize) -> Result<(), EraseError>;

    /// Starts writing `data` at `address`. The target must already be erased
    fn start_write(&mut self, address: usize, data: &[u8]) -> Result<(), WriteError>;

    /// [`WouldBlock`](nb::Error::WouldBlock) while an operation runs, then its result. `Ok` when nothing is in progress
    fn poll(&mut self) -> nb::Result<(), FlashError>;

    /// Indices of the sectors exactly covering `range`, for erasing with `start_erase_sector`
    fn sectors_in(&self, range: Range<usize>) -> Result<Range<usize>, EraseError> {
        if range.is_empty() {
            return Ok(0..0);
        }

        let sectors = self.get_sectors();
        let (Some(first), Some(last)) = (sectors.first(), sectors.last()) else {
            return Err(EraseError::OutOfRange);
        };
        if range.start < first.address || range.end > last.end() {
            return Err(EraseError::OutOfRange);
        }

        let start = sectors.iter().position(|s| s.address == range.start).ok_or(EraseError::Unaligned)?;
        let end = sectors.iter().position(|s| s.end() == range.end).ok_or(EraseError::Unaligned)?;
        Ok(start..end + 1)
    }
}

/// The smallest region of flash that can be erased
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sector {
    pub address: usize,
    pub size: usize,
}

impl Sector {
    /// First address past the sector
    pub const fn end(&self) -> usize {
        self.address + self.size
    }
}

/// Why an erase couldn't be started
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EraseError {
    /// Another operation is still running
    Busy,

    /// The sector index or address range is outside flash
    OutOfRange,

    /// The range doesn't start and end on sector boundaries
    Unaligned,

    /// The sector belongs to the bootloader or is otherwise off limits
    Protected,
}

/// Why a write couldn't be started
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteError {
    /// Another operation is still running
    Busy,

    /// The target is outside flash
    OutOfRange,

    /// The address or length isn't a multiple of `WRITE_ALIGN`
    Unaligned,

    /// `data` is longer than `MAX_WRITE`
    TooLarge,

    /// The target belongs to the bootloader or is otherwise off limits
    Protected,
}

/// Why a started operation failed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlashError {
    /// The hardware refused because the target is write protected
    WriteProtected,

    /// The hardware reported an erase or programming failure
    Operation,
}
