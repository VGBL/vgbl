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

    /// The flash layout, in ascending address order. Regions may have gaps between them
    fn get_regions(&self) -> &[Region];

    /// Starts erasing one sector. `sector` is a [`Sector::index`]
    fn start_erase_sector(&mut self, sector: usize) -> Result<(), EraseError>;

    /// Starts writing `data` at `address`
    ///
    /// Rejects anything the hardware can't program correctly, including a target that isn't erased
    fn start_write(&mut self, address: usize, data: &[u8]) -> Result<(), WriteError>;

    /// [`WouldBlock`](nb::Error::WouldBlock) while an operation runs, then its result. `Ok` when nothing is in progress
    fn poll(&mut self) -> nb::Result<(), FlashError>;

    /// The sector with this index
    fn sector(&self, index: usize) -> Option<Sector> {
        let mut first = 0;
        for region in self.get_regions() {
            if index < first + region.count {
                return Some(region.sector(first, index - first));
            }
            first += region.count;
        }
        None
    }

    /// The sector containing `address`
    fn sector_at(&self, address: usize) -> Option<Sector> {
        let mut first = 0;
        for region in self.get_regions() {
            if region.contains(address) {
                return Some(region.sector(first, (address - region.address) / region.sector_size));
            }
            first += region.count;
        }
        None
    }

    /// Every sector overlapping `size` bytes from `address`, in address order
    ///
    /// The first and last sectors may extend past the range, and the range may run into gaps or off the end of
    /// flash. Check the result covers the range when that matters
    fn sectors(&self, address: usize, size: usize) -> Sectors<'_> {
        Sectors::new(self.get_regions(), address, size)
    }
}

/// A run of equal-sized sectors
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub address: usize,
    pub sector_size: usize,
    pub count: usize,
}

impl Region {
    pub const fn size(&self) -> usize {
        self.sector_size * self.count
    }

    /// First address past the region
    pub const fn end(&self) -> usize {
        self.address + self.size()
    }

    pub const fn contains(&self, address: usize) -> bool {
        address >= self.address && address < self.end()
    }

    /// Sector `offset` of this region, where `first` is the index of the region's first sector
    const fn sector(&self, first: usize, offset: usize) -> Sector {
        Sector { index: first + offset, address: self.address + offset * self.sector_size, size: self.sector_size }
    }
}

/// The smallest region of flash that can be erased
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sector {
    /// Position across all regions, counting up from the lowest address
    pub index: usize,
    pub address: usize,
    pub size: usize,
}

impl Sector {
    /// First address past the sector
    pub const fn end(&self) -> usize {
        self.address + self.size
    }
}

/// Iterator returned by [`Flash::sectors`]
#[derive(Clone, Debug)]
pub struct Sectors<'a> {
    regions: &'a [Region],
    /// Index into `regions` of the next sector
    region: usize,
    /// Index of the next sector within its region
    offset: usize,
    /// Index of the first sector in `regions[region]`
    first: usize,
    end: usize,
}

impl<'a> Sectors<'a> {
    fn new(regions: &'a [Region], address: usize, size: usize) -> Self {
        let mut sectors = Self { regions, region: regions.len(), offset: 0, first: 0, end: address.saturating_add(size) };
        if size == 0 {
            return sectors;
        }

        // Skip regions entirely below the range, then sectors below it in the first region that isn't
        for (i, region) in regions.iter().enumerate() {
            if region.end() > address {
                sectors.region = i;
                sectors.offset = address.saturating_sub(region.address) / region.sector_size;
                break;
            }
            sectors.first += region.count;
        }
        sectors
    }
}

impl Iterator for Sectors<'_> {
    type Item = Sector;

    fn next(&mut self) -> Option<Sector> {
        loop {
            let region = self.regions.get(self.region)?;
            if self.offset == region.count {
                self.first += region.count;
                self.region += 1;
                self.offset = 0;
                continue;
            }

            let sector = region.sector(self.first, self.offset);
            if sector.address >= self.end {
                self.region = self.regions.len();
                return None;
            }
            self.offset += 1;
            return Some(sector);
        }
    }
}

/// Why an erase couldn't be started
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EraseError {
    /// Another operation is still running
    Busy,

    /// No sector has this index
    OutOfRange,

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

    /// Part of the target isn't erased
    NotErased,

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

#[cfg(test)]
mod tests {
    use super::*;

    struct Layout(&'static [Region]);

    impl Flash for Layout {
        const MAX_WRITE: usize = 0;
        const WRITE_ALIGN: usize = 1;

        fn get_regions(&self) -> &[Region] {
            self.0
        }

        fn start_erase_sector(&mut self, _sector: usize) -> Result<(), EraseError> {
            unimplemented!()
        }

        fn start_write(&mut self, _address: usize, _data: &[u8]) -> Result<(), WriteError> {
            unimplemented!()
        }

        fn poll(&mut self) -> nb::Result<(), FlashError> {
            unimplemented!()
        }
    }

    /// STM32F446: 4x16K, 1x64K, 3x128K
    const F446: Layout = Layout(&[
        Region { address: 0x0800_0000, sector_size: 0x4000, count: 4 },
        Region { address: 0x0801_0000, sector_size: 0x1_0000, count: 1 },
        Region { address: 0x0802_0000, sector_size: 0x2_0000, count: 3 },
    ]);

    /// Two banks with a gap between them
    const GAPPED: Layout = Layout(&[
        Region { address: 0x1000, sector_size: 0x100, count: 2 },
        Region { address: 0x2000, sector_size: 0x100, count: 2 },
    ]);

    fn addresses(sectors: Sectors) -> Vec<(usize, usize)> {
        sectors.map(|s| (s.index, s.address)).collect()
    }

    #[test]
    fn sector_by_index() {
        assert_eq!(F446.sector(0), Some(Sector { index: 0, address: 0x0800_0000, size: 0x4000 }));
        assert_eq!(F446.sector(4), Some(Sector { index: 4, address: 0x0801_0000, size: 0x1_0000 }));
        assert_eq!(F446.sector(7), Some(Sector { index: 7, address: 0x0806_0000, size: 0x2_0000 }));
        assert_eq!(F446.sector(8), None);
    }

    #[test]
    fn sector_by_address() {
        assert_eq!(F446.sector_at(0x0800_0000).map(|s| s.index), Some(0));
        assert_eq!(F446.sector_at(0x0800_7FFF).map(|s| s.index), Some(1));
        assert_eq!(F446.sector_at(0x0802_0000).map(|s| s.index), Some(5));
        assert_eq!(F446.sector_at(0x0807_FFFF).map(|s| s.index), Some(7));
        assert_eq!(F446.sector_at(0x0808_0000), None);
        assert_eq!(F446.sector_at(0x07FF_FFFF), None);
        assert_eq!(GAPPED.sector_at(0x1800), None);
        assert_eq!(GAPPED.sector_at(0x2100).map(|s| s.index), Some(3));
    }

    #[test]
    fn sectors_in_range() {
        // Exact sectors
        assert_eq!(addresses(F446.sectors(0x0800_4000, 0x8000)), [(1, 0x0800_4000), (2, 0x0800_8000)]);
        // Partial sectors at both ends, across a region boundary
        assert_eq!(
            addresses(F446.sectors(0x0800_C100, 0x1_4000)),
            [(3, 0x0800_C000), (4, 0x0801_0000), (5, 0x0802_0000)]
        );
        // Starting before flash, running off the end
        assert_eq!(addresses(F446.sectors(0x0700_0000, 0x0100_4000)), [(0, 0x0800_0000)]);
        assert_eq!(addresses(F446.sectors(0x0806_0000, usize::MAX)), [(7, 0x0806_0000)]);
        // Across a gap
        assert_eq!(addresses(GAPPED.sectors(0x1100, 0x1000)), [(1, 0x1100), (2, 0x2000)]);
        // Nothing
        assert_eq!(addresses(F446.sectors(0x0800_0000, 0)), []);
        assert_eq!(addresses(F446.sectors(0x0808_0000, 0x100)), []);
        assert_eq!(addresses(GAPPED.sectors(0x1400, 0x100)), []);
    }
}
