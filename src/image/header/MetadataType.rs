/// Metadata entries exist in a segment header to describe some fundamental trait of the payload

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum MetadataType {
    /// Marks segment as executable and defines an entry point
    Executable = 0x0001,

    /// Defines a generic information key (name, author, etc)
    Info = 0x0002,

    /// Defines a cryptographic validation method, expected cryptographic result, and validation triggers to validate against the payload
    /// * Expected result can be a raw value or a reference to the internal storage location of a key
    Validation = 0x0003,

    /// Defines a version identifier
    Version = 0x0004,

    /// Defines the anti-rollback version
    Rollback = 0x0005,
}

