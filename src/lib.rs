#![cfg_attr(not(test), no_std)]

mod bootloader;
pub mod hal;
mod hooks;

pub use bootloader::Bootloader;
pub use hooks::Hooks;

pub mod image {
    mod encode;
    mod entry;
    mod parse;

    pub use encode::{EncodeError, Encoded, HeaderWriter};
    pub use entry::{
        CRITICAL, ChecksumAlgorithm, ChecksumEntry, Entry, EntryKind, HashAlgorithm, HashEntry, LoadEntry, LoadFlags,
        Version,
    };
    pub use parse::{Entries, FOOTER_MAGIC, FORMAT_MAJOR, FORMAT_MINOR, Header, MAGIC, ParseError};

    /// The fixed header copyup reads until it moves to [`Header`]
    pub mod header {
        mod image_header;

        pub use image_header::ImageHeader;
    }
}

pub mod validation {
    mod validation_triggers;

    pub use validation_triggers::ValidationTriggers;
}
