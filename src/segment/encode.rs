use crate::segment::parse::{
    ALIGN, DIGEST_LEN, ENTRY_HEADER_LEN, FOOTER_LEN, FOOTER_MAGIC, FOOTER_PREFIX_LEN, FORMAT_MAJOR, FORMAT_MINOR, MAGIC,
    STATIC_LEN, pad,
};
use crate::segment::{Entry, Header, ParseError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncodeError {
    /// The header doesn't fit in the buffer
    BufferFull,
    /// More entries than the count field holds
    TooManyEntries,
    /// A value is longer than the length field holds
    ValueTooLong,
    /// The data alignment isn't a power of two
    BadAlignment,
    /// The finished header doesn't parse, e.g. a duplicate or an out-of-range entry
    Invalid(ParseError),
}

/// Builds a segment header in a caller-provided buffer
///
/// ```ignore
/// let mut writer = HeaderWriter::new(&mut buffer, data.len() as u32);
/// writer.push(&Entry::Name("app"))?;
/// writer.push(&Entry::Jump(0x0800_4200))?;
/// let header = writer.finish(0, 512, sha256)?;
/// // Flash header.bytes(), then pad to header.data_offset(), then the data
/// ```
pub struct HeaderWriter<'b> {
    buffer: &'b mut [u8],
    position: usize,
    entry_count: u16,
    data_len: u32,
}

impl<'b> HeaderWriter<'b> {
    pub fn new(buffer: &'b mut [u8], data_len: u32) -> Self {
        Self { buffer, position: STATIC_LEN, entry_count: 0, data_len }
    }

    /// Appends an entry. Entries are written in the order pushed
    pub fn push(&mut self, entry: &Entry) -> Result<(), EncodeError> {
        let len = entry.encoded_len();
        let len_field = u16::try_from(len).map_err(|_| EncodeError::ValueTooLong)?;
        let count = self.entry_count.checked_add(1).ok_or(EncodeError::TooManyEntries)?;

        let start = self.position + ENTRY_HEADER_LEN;
        let next = pad(start + len);
        let slot = self.buffer.get_mut(self.position..next).ok_or(EncodeError::BufferFull)?;
        slot[0..2].copy_from_slice(&entry.kind().to_le_bytes());
        slot[2..4].copy_from_slice(&len_field.to_le_bytes());
        entry.encode(&mut slot[ENTRY_HEADER_LEN..ENTRY_HEADER_LEN + len]);
        slot[ENTRY_HEADER_LEN + len..].fill(0);

        self.position = next;
        self.entry_count = count;
        Ok(())
    }

    /// Writes the static header and footer, then checks the result parses
    ///
    /// * `signature_len`: room reserved for a signature, filled in afterwards through [`Encoded::signature_mut`]
    /// * `data_align`: the data offset is rounded up to a multiple of this, e.g. for vector table alignment
    /// * `sha256`: computes the digest of the bytes passed to it
    pub fn finish(
        self,
        signature_len: u16,
        data_align: u32,
        sha256: impl FnOnce(&[u8]) -> [u8; DIGEST_LEN],
    ) -> Result<Encoded<'b>, EncodeError> {
        if !data_align.is_power_of_two() {
            return Err(EncodeError::BadAlignment);
        }

        let footer = self.position;
        let signature = footer + FOOTER_LEN;
        let end = pad(signature + signature_len as usize);
        let data_offset = u32::try_from(end.next_multiple_of(data_align.max(ALIGN as u32) as usize))
            .map_err(|_| EncodeError::BufferFull)?;
        let buffer = self.buffer.get_mut(..end).ok_or(EncodeError::BufferFull)?;

        buffer[0..4].copy_from_slice(&MAGIC);
        buffer[4] = FORMAT_MAJOR;
        buffer[5] = FORMAT_MINOR;
        buffer[6..8].copy_from_slice(&self.entry_count.to_le_bytes());
        buffer[8..12].copy_from_slice(&data_offset.to_le_bytes());
        buffer[12..16].copy_from_slice(&self.data_len.to_le_bytes());

        buffer[footer..footer + 4].copy_from_slice(&FOOTER_MAGIC);
        buffer[footer + 4..footer + 6].copy_from_slice(&signature_len.to_le_bytes());
        buffer[footer + 6..footer + 8].fill(0);
        buffer[signature..].fill(0);

        let digest = sha256(&buffer[..footer + FOOTER_PREFIX_LEN]);
        buffer[footer + FOOTER_PREFIX_LEN..signature].copy_from_slice(&digest);

        Header::parse(buffer).map_err(EncodeError::Invalid)?;
        Ok(Encoded { bytes: buffer, footer, signature_len: signature_len as usize, data_offset })
    }
}

/// A finished header
pub struct Encoded<'b> {
    bytes: &'b mut [u8],
    footer: usize,
    signature_len: usize,
    data_offset: u32,
}

impl Encoded<'_> {
    /// The whole header, to write at the start of the sector
    pub fn bytes(&self) -> &[u8] {
        self.bytes
    }

    /// Where the segment data goes, from the header start. The gap after the header is left to the caller
    pub fn data_offset(&self) -> u32 {
        self.data_offset
    }

    /// The digest to sign
    pub fn digest(&self) -> &[u8; DIGEST_LEN] {
        let start = self.footer + FOOTER_PREFIX_LEN;
        self.bytes[start..start + DIGEST_LEN].try_into().unwrap()
    }

    /// The reserved signature space. Not covered by the digest, so it can be written after `finish`
    pub fn signature_mut(&mut self) -> &mut [u8] {
        let start = self.footer + FOOTER_LEN;
        &mut self.bytes[start..start + self.signature_len]
    }
}
