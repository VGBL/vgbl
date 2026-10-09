/// Describes where a segment is loaded and how to start it
///
/// Stored little-endian, immediately before the segment data

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct CopyupHeader {
    /// Address the payload is copied to before execution
    pub load_addr: u32,

    /// Payload size in bytes
    pub size: u32,

    /// Offset of the vector table from `load_addr`
    pub vector_table_offset: u32,
}

impl CopyupHeader {
    /// Serialized size in bytes
    pub const SIZE: usize = 12;

    pub const fn from_bytes(bytes: &[u8; Self::SIZE]) -> Self {
        Self {
            load_addr: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            size: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            vector_table_offset: u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
        }
    }

    pub const fn to_bytes(&self) -> [u8; Self::SIZE] {
        let load_addr = self.load_addr.to_le_bytes();
        let size = self.size.to_le_bytes();
        let vector_table_offset = self.vector_table_offset.to_le_bytes();
        [
            load_addr[0], load_addr[1], load_addr[2], load_addr[3],
            size[0], size[1], size[2], size[3],
            vector_table_offset[0], vector_table_offset[1], vector_table_offset[2], vector_table_offset[3],
        ]
    }
}
