use crate::image::ParseError;
use crate::validation::ValidationTriggers;

/// Set on entry kinds a parser must understand. Unknown kinds without it are passed through as
/// [`Entry::Unknown`], unknown kinds with it make the header invalid
pub const CRITICAL: u16 = 0x8000;

/// Every entry kind this version of the format defines
///
/// `0x00xx` descriptors, `0x81xx` execution, `0x82xx` validation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum EntryKind {
    Name = 0x0001,
    Company = 0x0002,
    Version = 0x0003,
    Branch = 0x0004,
    UserId = 0x0005,
    BuildDate = 0x0006,

    Bootloader = 0x8101,
    Target = 0x8102,
    Load = 0x8103,
    TargetSpecific = 0x8104,
    Jump = 0x8105,

    Checksum = 0x8201,
    Hash = 0x8202,
}

impl EntryKind {
    pub const fn from_u16(kind: u16) -> Option<Self> {
        Some(match kind {
            0x0001 => Self::Name,
            0x0002 => Self::Company,
            0x0003 => Self::Version,
            0x0004 => Self::Branch,
            0x0005 => Self::UserId,
            0x0006 => Self::BuildDate,
            0x8101 => Self::Bootloader,
            0x8102 => Self::Target,
            0x8103 => Self::Load,
            0x8104 => Self::TargetSpecific,
            0x8105 => Self::Jump,
            0x8201 => Self::Checksum,
            0x8202 => Self::Hash,
            _ => return None,
        })
    }

    pub const fn is_critical(self) -> bool {
        self as u16 & CRITICAL != 0
    }

    /// Whether a header may contain this kind at most once
    pub const fn is_unique(self) -> bool {
        !matches!(self, Self::Load | Self::TargetSpecific | Self::Checksum | Self::Hash)
    }

    /// A distinct bit per kind, for tracking which have been seen
    pub(crate) const fn bit(self) -> u16 {
        1 << match self {
            Self::Name => 0,
            Self::Company => 1,
            Self::Version => 2,
            Self::Branch => 3,
            Self::UserId => 4,
            Self::BuildDate => 5,
            Self::Bootloader => 6,
            Self::Target => 7,
            Self::Load => 8,
            Self::TargetSpecific => 9,
            Self::Jump => 10,
            Self::Checksum => 11,
            Self::Hash => 12,
        }
    }
}

/// One decoded header entry. Offsets are relative to the start of the segment data
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry<'a> {
    Name(&'a str),
    Company(&'a str),
    Version(Version),
    Branch(&'a str),
    /// Assigned by the user to identify the segment
    UserId([u8; 16]),
    /// Unix time in seconds
    BuildDate(u64),

    /// The segment is a bootloader, run at this stage
    Bootloader { stage: u8 },
    /// The chip the segment is built for. Must match the HAL before anything runs
    Target(&'a str),
    Load(LoadEntry),
    /// Chip-specific execution setup, interpreted by the HAL. Only valid alongside [`Entry::Target`]
    TargetSpecific(&'a [u8]),
    /// Address to jump to once everything else is done. Its presence makes the segment executable
    Jump(u32),

    Checksum(ChecksumEntry),
    Hash(HashEntry<'a>),

    /// A non-critical kind this version doesn't know
    Unknown { kind: u16, value: &'a [u8] },
}

/// Semantic version of the segment contents
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

/// Copy part of the segment data to `address` before execution
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoadEntry {
    pub offset: u32,
    pub len: u32,
    pub address: u32,
    pub flags: LoadFlags,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoadFlags(u32);

impl LoadFlags {
    pub const NONE: Self = Self(0);
    /// Check the copy against the source once it's written
    pub const VERIFY: Self = Self(0x0000_0001);

    const ALL: u32 = Self::VERIFY.0;

    /// `None` if any bit is undefined
    pub const fn from_bits(bits: u32) -> Option<Self> {
        if bits & !Self::ALL == 0 { Some(Self(bits)) } else { None }
    }

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ChecksumAlgorithm {
    Crc32 = 1,
}

impl ChecksumAlgorithm {
    pub const fn from_u8(id: u8) -> Option<Self> {
        match id {
            1 => Some(Self::Crc32),
            _ => None,
        }
    }
}

/// A checksum over part of the segment data, for detecting corruption
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChecksumEntry {
    pub algorithm: ChecksumAlgorithm,
    pub triggers: ValidationTriggers,
    pub offset: u32,
    pub len: u32,
    pub value: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum HashAlgorithm {
    Sha256 = 1,
}

impl HashAlgorithm {
    pub const fn from_u8(id: u8) -> Option<Self> {
        match id {
            1 => Some(Self::Sha256),
            _ => None,
        }
    }

    pub const fn digest_len(self) -> usize {
        match self {
            Self::Sha256 => 32,
        }
    }
}

/// A hash over part of the segment data. Covered by the header signature, so it authenticates the data
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HashEntry<'a> {
    pub algorithm: HashAlgorithm,
    pub triggers: ValidationTriggers,
    pub offset: u32,
    pub len: u32,
    /// [`HashAlgorithm::digest_len`] bytes
    pub digest: &'a [u8],
}

/// Every trigger bit the format defines
const TRIGGERS: u16 =
    ValidationTriggers::COLD_BOOT.union(ValidationTriggers::WARM_BOOT).union(ValidationTriggers::UPDATE).bits();

/// `algorithm u8, reserved u8, triggers u16, offset u32, len u32`
const VALIDATION_PREFIX: usize = 12;

impl<'a> Entry<'a> {
    pub const fn kind(&self) -> u16 {
        (match self {
            Self::Name(_) => EntryKind::Name,
            Self::Company(_) => EntryKind::Company,
            Self::Version(_) => EntryKind::Version,
            Self::Branch(_) => EntryKind::Branch,
            Self::UserId(_) => EntryKind::UserId,
            Self::BuildDate(_) => EntryKind::BuildDate,
            Self::Bootloader { .. } => EntryKind::Bootloader,
            Self::Target(_) => EntryKind::Target,
            Self::Load(_) => EntryKind::Load,
            Self::TargetSpecific(_) => EntryKind::TargetSpecific,
            Self::Jump(_) => EntryKind::Jump,
            Self::Checksum(_) => EntryKind::Checksum,
            Self::Hash(_) => EntryKind::Hash,
            Self::Unknown { kind, .. } => return *kind,
        }) as u16
    }

    /// The `(offset, len)` of segment data the entry refers to, if any
    pub const fn data_range(&self) -> Option<(u32, u32)> {
        match self {
            Self::Load(e) => Some((e.offset, e.len)),
            Self::Checksum(e) => Some((e.offset, e.len)),
            Self::Hash(e) => Some((e.offset, e.len)),
            _ => None,
        }
    }

    pub(crate) fn decode(kind: u16, value: &'a [u8]) -> Result<Self, ParseError> {
        let Some(known) = EntryKind::from_u16(kind) else {
            return if kind & CRITICAL != 0 { Err(ParseError::UnknownCritical { kind }) } else { Ok(Self::Unknown { kind, value }) };
        };
        let bad_length = ParseError::BadLength { kind };
        let string = |value| core::str::from_utf8(value).map_err(|_| ParseError::BadString { kind });

        Ok(match known {
            EntryKind::Name => Self::Name(string(value)?),
            EntryKind::Company => Self::Company(string(value)?),
            EntryKind::Branch => Self::Branch(string(value)?),
            EntryKind::Target => Self::Target(string(value)?),
            EntryKind::Version => {
                let v: &[u8; 6] = value.try_into().map_err(|_| bad_length)?;
                Self::Version(Version { major: u16_at(v, 0), minor: u16_at(v, 2), patch: u16_at(v, 4) })
            }
            EntryKind::UserId => Self::UserId(*<&[u8; 16]>::try_from(value).map_err(|_| bad_length)?),
            EntryKind::BuildDate => Self::BuildDate(u64::from_le_bytes(*<&[u8; 8]>::try_from(value).map_err(|_| bad_length)?)),
            EntryKind::Bootloader => match value {
                [stage] => Self::Bootloader { stage: *stage },
                _ => return Err(bad_length),
            },
            EntryKind::Load => {
                let v: &[u8; 16] = value.try_into().map_err(|_| bad_length)?;
                let flags = LoadFlags::from_bits(u32_at(v, 12)).ok_or(ParseError::UnknownFlags { kind })?;
                Self::Load(LoadEntry { offset: u32_at(v, 0), len: u32_at(v, 4), address: u32_at(v, 8), flags })
            }
            EntryKind::TargetSpecific => Self::TargetSpecific(value),
            EntryKind::Jump => Self::Jump(u32::from_le_bytes(*<&[u8; 4]>::try_from(value).map_err(|_| bad_length)?)),
            EntryKind::Checksum => {
                let v: &[u8; VALIDATION_PREFIX + 4] = value.try_into().map_err(|_| bad_length)?;
                let algorithm = ChecksumAlgorithm::from_u8(v[0]).ok_or(ParseError::UnknownAlgorithm { kind })?;
                Self::Checksum(ChecksumEntry {
                    algorithm,
                    triggers: triggers(kind, v)?,
                    offset: u32_at(v, 4),
                    len: u32_at(v, 8),
                    value: u32_at(v, 12),
                })
            }
            EntryKind::Hash => {
                let prefix = value.get(..VALIDATION_PREFIX).ok_or(bad_length)?;
                let algorithm = HashAlgorithm::from_u8(prefix[0]).ok_or(ParseError::UnknownAlgorithm { kind })?;
                let digest = &value[VALIDATION_PREFIX..];
                if digest.len() != algorithm.digest_len() {
                    return Err(bad_length);
                }
                Self::Hash(HashEntry {
                    algorithm,
                    triggers: triggers(kind, prefix)?,
                    offset: u32_at(prefix, 4),
                    len: u32_at(prefix, 8),
                    digest,
                })
            }
        })
    }

    /// Length of the encoded value, before padding
    pub(crate) const fn encoded_len(&self) -> usize {
        match self {
            Self::Name(s) | Self::Company(s) | Self::Branch(s) | Self::Target(s) => s.len(),
            Self::Version(_) => 6,
            Self::UserId(_) => 16,
            Self::BuildDate(_) => 8,
            Self::Bootloader { .. } => 1,
            Self::Load(_) => 16,
            Self::TargetSpecific(value) | Self::Unknown { value, .. } => value.len(),
            Self::Jump(_) => 4,
            Self::Checksum(_) => VALIDATION_PREFIX + 4,
            Self::Hash(e) => VALIDATION_PREFIX + e.digest.len(),
        }
    }

    /// Writes the value into `out`, which is exactly `encoded_len` bytes
    pub(crate) fn encode(&self, out: &mut [u8]) {
        match self {
            Self::Name(s) | Self::Company(s) | Self::Branch(s) | Self::Target(s) => out.copy_from_slice(s.as_bytes()),
            Self::Version(v) => {
                out[0..2].copy_from_slice(&v.major.to_le_bytes());
                out[2..4].copy_from_slice(&v.minor.to_le_bytes());
                out[4..6].copy_from_slice(&v.patch.to_le_bytes());
            }
            Self::UserId(id) => out.copy_from_slice(id),
            Self::BuildDate(seconds) => out.copy_from_slice(&seconds.to_le_bytes()),
            Self::Bootloader { stage } => out[0] = *stage,
            Self::Load(e) => {
                out[0..4].copy_from_slice(&e.offset.to_le_bytes());
                out[4..8].copy_from_slice(&e.len.to_le_bytes());
                out[8..12].copy_from_slice(&e.address.to_le_bytes());
                out[12..16].copy_from_slice(&e.flags.bits().to_le_bytes());
            }
            Self::TargetSpecific(value) | Self::Unknown { value, .. } => out.copy_from_slice(value),
            Self::Jump(address) => out.copy_from_slice(&address.to_le_bytes()),
            Self::Checksum(e) => {
                encode_validation(out, e.algorithm as u8, e.triggers, e.offset, e.len);
                out[12..16].copy_from_slice(&e.value.to_le_bytes());
            }
            Self::Hash(e) => {
                encode_validation(out, e.algorithm as u8, e.triggers, e.offset, e.len);
                out[VALIDATION_PREFIX..].copy_from_slice(e.digest);
            }
        }
    }
}

fn encode_validation(out: &mut [u8], algorithm: u8, triggers: ValidationTriggers, offset: u32, len: u32) {
    out[0] = algorithm;
    out[1] = 0;
    out[2..4].copy_from_slice(&triggers.bits().to_le_bytes());
    out[4..8].copy_from_slice(&offset.to_le_bytes());
    out[8..12].copy_from_slice(&len.to_le_bytes());
}

fn triggers(kind: u16, prefix: &[u8]) -> Result<ValidationTriggers, ParseError> {
    let bits = u16_at(prefix, 2);
    if bits & !TRIGGERS != 0 {
        return Err(ParseError::UnknownFlags { kind });
    }
    Ok(ValidationTriggers::from_bits(bits))
}

pub(crate) fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

pub(crate) fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}
