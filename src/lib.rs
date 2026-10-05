#![cfg_attr(not(test), no_std)]

mod bootloader;
pub mod hal;

pub use bootloader::Bootloader;

pub mod image {
    pub mod header {
        mod image_header;
        mod metadata_type;

        pub use image_header::ImageHeader;
        pub use metadata_type::MetadataType;
    }
}
