//  SPEC.rs
//    by Lut99
// 
//  Created:
//    29 Apr 2022, 18:16:49
//  Last edited:
//    13 Aug 2022, 16:22:32
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines traits and other public interfaces in the Vulkan crate.
// 

use ash::vk;
use semver::Version;


/***** LIBRARY *****/
/// Defines Vulkan-compatible API Version numbers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiVersion {
    pub variant : u8,
    pub major   : u8,
    pub minor   : u16,
    pub patch   : u16,
}

impl ApiVersion {
    /// Defines api version 1.0
    pub const VK_1_0: Self = Self::new(1, 0, 0);
    /// Defines api version 1.1
    pub const VK_1_1: Self = Self::new(1, 1, 0);
    /// Defines api version 1.2
    pub const VK_1_2: Self = Self::new(1, 2, 0);
    /// Defines api version 1.3
    pub const VK_1_3: Self = Self::new(1, 3, 0);



    /// Constructor for the ApiVersion.
    /// 
    /// # Arguments
    /// - `major`: The major version number. Note that only the 7 least significant bits will be used.
    /// - `minor`: The minor version number. Note that only the 10 least significant bits will be used.
    /// - `patch`: The patch version number. Note that only the 12 least significant bits will be used.
    /// 
    /// # Returns
    /// A new ApiVersion instance.
    #[inline]
    pub const fn new(major: u8, minor: u16, patch: u16) -> Self {
        Self {
            variant : 0,
            major,
            minor,
            patch,
        }
    }

    /// Constructor for the ApiVersion that takes a custom variant.
    /// 
    /// # Arguments
    /// - `variant`: The variant identifier. Note that only the 5 least significant bits will be used.
    /// - `major`: The major version number. Note that only the 7 least significant bits will be used.
    /// - `minor`: The minor version number. Note that only the 10 least significant bits will be used.
    /// - `patch`: The patch version number. Note that only the 12 least significant bits will be used.
    /// 
    /// # Returns
    /// A new ApiVersion instance.
    #[inline]
    pub const fn new_with_variant(variant: u8, major: u8, minor: u16, patch: u16) -> Self {
        Self {
            variant,
            major,
            minor,
            patch,
        }
    }
}

impl From<u32> for ApiVersion {
    #[inline]
    fn from(value: u32) -> Self {
        Self {
            variant : vk::api_version_variant(value) as u8,
            major   : vk::api_version_major(value) as u8,
            minor   : vk::api_version_minor(value) as u16,
            patch   : vk::api_version_patch(value) as u16,
        }
    }
}

impl From<ApiVersion> for u32 {
    fn from(value: ApiVersion) -> Self {
        // Do sanity checks
        if (value.variant & (!0x1F))  != 0 { panic!("ApiVersion.variant cannot be converted to a 5-bit value ({:#X})", value.variant); }
        if (value.major   & (!0x7F))  != 0 { panic!("ApiVersion.major cannot be converted to a 7-bit value ({:#X})" , value.major); }
        if (value.minor   & (!0x3FF)) != 0 { panic!("ApiVersion.minor cannot be converted to a 10-bit value ({:#X})", value.minor); }
        if (value.patch   & (!0xFFF)) != 0 { panic!("ApiVersion.patch cannot be converted to a 12-bit value ({:#X})", value.patch); }

        // Return the integer
        vk::make_api_version(value.variant as u32, value.major as u32, value.minor as u32, value.patch as u32)
    }
}

impl From<Version> for ApiVersion {
    #[inline]
    fn from(value: Version) -> Self {
        Self::new(value.major as u8, value.minor as u16, value.patch as u16)
    }
}

impl From<ApiVersion> for Version {
    #[inline]
    fn from(value: ApiVersion) -> Self {
        Self::new(value.major as u64, value.minor as u64, value.patch as u64)
    }
}



/// Defines Vulkan-compatible device driver numbers.
/// 
/// Note that these differ per vendor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriverVersion(u32);

impl DriverVersion {
    /* TBD: Once we need it, we add vendor-specific parses and encoders here. */
}

impl From<u32> for DriverVersion {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<DriverVersion> for u32 {
    #[inline]
    fn from(value: DriverVersion) -> Self {
        value.0
    }
}
