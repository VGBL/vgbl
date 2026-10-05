#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValidationTriggers(u16);
 
impl ValidationTriggers {
    pub const NONE: Self = Self(0x0000);
    pub const COLD_BOOT: Self = Self(0x0001);
    pub const WARM_BOOT: Self = Self(0x0002);
    pub const UPDATE: Self = Self(0x0004);
 
    pub const fn from_bits(bits: u16) -> Self {
        Self(bits)
    }
 
    pub const fn bits(self) -> u16 {
        self.0
    }
 
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
 
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}