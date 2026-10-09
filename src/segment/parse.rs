/*
    Segment header format, all little-endian

    A header sits at the start of a flash sector and wraps the segment data that follows it.

    Static header (16 bytes)
        0   magic           "VGBL"
        4   format major    u8      parsers reject any other major
        5   format minor    u8      newer minors only add non-critical entries
        6   entry count     u16
        8   data offset     u32     from the header start to the segment data, past the footer
        12  data length     u32

    Entries, `entry count` of them, each padded with zeros to a multiple of 16 bytes
        0   kind            u16     bit 15 set: a parser that doesn't know the kind must reject the header
        2   value length    u16     before padding
        4   value

    Footer
        0   magic           "RMFT"
        4   signature len   u16     0 when unsigned
        6   reserved        u16
        8   digest          [u8; 32]    SHA-256 of everything from the header start up to the digest
        40  signature               signs the digest
        ..  zero padding to a multiple of 16 bytes
*/

use crate::segment::entry::{u16_at, u32_at};
use crate::segment::{Entry, EntryKind};

pub const MAGIC: [u8; 4] = *b"VGBL";
pub const FOOTER_MAGIC: [u8; 4] = *b"RMFT";

pub const FORMAT_MAJOR: u8 = 0;
pub const FORMAT_MINOR: u8 = 1;

pub(crate) const STATIC_LEN: usize = 16;
pub(crate) const ENTRY_HEADER_LEN: usize = 4;
/// Entries and the footer each occupy a multiple of this
pub(crate) const ALIGN: usize = 16;
pub(crate) const DIGEST_LEN: usize = 32;
/// Footer up to the digest: magic, signature length, reserved
pub(crate) const FOOTER_PREFIX_LEN: usize = 8;
/// Footer without the signature or padding
pub(crate) const FOOTER_LEN: usize = FOOTER_PREFIX_LEN + DIGEST_LEN;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    /// The bytes end before the header does
    Truncated,
    BadMagic,
    BadFooterMagic,
    UnsupportedVersion { major: u8 },
    /// The value is the wrong length for its kind
    BadLength { kind: u16 },
    /// A string value isn't UTF-8
    BadString { kind: u16 },
    UnknownCritical { kind: u16 },
    UnknownAlgorithm { kind: u16 },
    UnknownFlags { kind: u16 },
    /// A kind that may appear once appears again
    Duplicate { kind: u16 },
    /// An entry refers to data past the end of the segment
    OutOfRange { kind: u16 },
    TargetSpecificWithoutTarget,
    /// The data offset points inside the header
    DataOverlapsHeader,
}

/// A parsed segment header
///
/// Parsing checks the structure: magics, version, every entry's encoding, uniqueness and ranges. It doesn't check the
/// digest or signature
#[derive(Clone, Copy, Debug)]
pub struct Header<'a> {
    /// The whole header, footer and padding included
    bytes: &'a [u8],
    minor: u8,
    entry_count: u16,
    data_offset: u32,
    data_len: u32,
    footer: usize,
    signature_len: usize,
}

impl<'a> Header<'a> {
    /// Parses the header at the start of `bytes`. Anything past the header is ignored, so on memory-mapped flash
    /// this can be the whole sector
    pub fn parse(bytes: &'a [u8]) -> Result<Self, ParseError> {
        let fixed = bytes.get(..STATIC_LEN).ok_or(ParseError::Truncated)?;
        if fixed[0..4] != MAGIC {
            return Err(ParseError::BadMagic);
        }
        if fixed[4] != FORMAT_MAJOR {
            return Err(ParseError::UnsupportedVersion { major: fixed[4] });
        }
        let minor = fixed[5];
        let entry_count = u16_at(fixed, 6);
        let data_offset = u32_at(fixed, 8);
        let data_len = u32_at(fixed, 12);

        let mut position = STATIC_LEN;
        let mut seen = 0;
        let mut has_target = false;
        let mut has_target_specific = false;
        for _ in 0..entry_count {
            let (kind, value, next) = read_entry(bytes, position)?;
            let entry = Entry::decode(kind, value)?;

            if let Some(known) = EntryKind::from_u16(kind) {
                if known.is_unique() && seen & known.bit() != 0 {
                    return Err(ParseError::Duplicate { kind });
                }
                seen |= known.bit();
                has_target |= known == EntryKind::Target;
                has_target_specific |= known == EntryKind::TargetSpecific;
            }
            if let Some((offset, len)) = entry.data_range()
                && offset.checked_add(len).is_none_or(|end| end > data_len)
            {
                return Err(ParseError::OutOfRange { kind });
            }

            position = next;
        }
        if has_target_specific && !has_target {
            return Err(ParseError::TargetSpecificWithoutTarget);
        }

        let footer = position;
        let prefix = bytes.get(footer..footer + FOOTER_LEN).ok_or(ParseError::Truncated)?;
        if prefix[0..4] != FOOTER_MAGIC {
            return Err(ParseError::BadFooterMagic);
        }
        let signature_len = u16_at(prefix, 4) as usize;
        let end = pad(footer + FOOTER_LEN + signature_len);
        let bytes = bytes.get(..end).ok_or(ParseError::Truncated)?;
        if (data_offset as usize) < end {
            return Err(ParseError::DataOverlapsHeader);
        }

        Ok(Self { bytes, minor, entry_count, data_offset, data_len, footer, signature_len })
    }

    /// The whole header, footer and padding included
    pub fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }

    /// `(major, minor)` of the header format, not the segment contents
    pub fn format_version(&self) -> (u8, u8) {
        (FORMAT_MAJOR, self.minor)
    }

    pub fn entry_count(&self) -> u16 {
        self.entry_count
    }

    pub fn entries(&self) -> Entries<'a> {
        Entries { bytes: self.bytes, position: STATIC_LEN, remaining: self.entry_count }
    }

    /// From the header start to the segment data
    pub fn data_offset(&self) -> u32 {
        self.data_offset
    }

    pub fn data_len(&self) -> u32 {
        self.data_len
    }

    /// Whether the segment can be jumped to
    pub fn is_executable(&self) -> bool {
        self.entries().any(|e| matches!(e, Entry::Jump(_)))
    }

    /// The bytes `digest` covers
    pub fn hashed(&self) -> &'a [u8] {
        &self.bytes[..self.footer + FOOTER_PREFIX_LEN]
    }

    /// SHA-256 of [`hashed`](Self::hashed), as stored
    pub fn digest(&self) -> &'a [u8; DIGEST_LEN] {
        let start = self.footer + FOOTER_PREFIX_LEN;
        self.bytes[start..start + DIGEST_LEN].try_into().unwrap()
    }

    pub fn signature(&self) -> Option<&'a [u8]> {
        let start = self.footer + FOOTER_LEN;
        (self.signature_len > 0).then(|| &self.bytes[start..start + self.signature_len])
    }
}

/// Iterator over a parsed header's entries, in order
#[derive(Clone, Debug)]
pub struct Entries<'a> {
    bytes: &'a [u8],
    position: usize,
    remaining: u16,
}

impl<'a> Iterator for Entries<'a> {
    type Item = Entry<'a>;

    fn next(&mut self) -> Option<Entry<'a>> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;

        // Parsing already checked every entry decodes
        let (kind, value, next) = read_entry(self.bytes, self.position).ok()?;
        self.position = next;
        Entry::decode(kind, value).ok()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining as usize, Some(self.remaining as usize))
    }
}

/// The kind, value and next entry's position of the entry at `position`
fn read_entry(bytes: &[u8], position: usize) -> Result<(u16, &[u8], usize), ParseError> {
    let header = bytes.get(position..position + ENTRY_HEADER_LEN).ok_or(ParseError::Truncated)?;
    let kind = u16_at(header, 0);
    let len = u16_at(header, 2) as usize;

    let start = position + ENTRY_HEADER_LEN;
    let next = pad(start + len);
    if next > bytes.len() {
        return Err(ParseError::Truncated);
    }
    Ok((kind, &bytes[start..start + len], next))
}

/// Rounds up to a multiple of `ALIGN`
pub(crate) const fn pad(len: usize) -> usize {
    len.next_multiple_of(ALIGN)
}
