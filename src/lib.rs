//  LIB.rs
//    by Lut99
// 
//  Created:
//    26 Mar 2022, 14:09:20
//  Last edited:
//    06 Aug 2022, 16:08:42
//  Auto updated?
//    Yes
// 
//  Description:
//!   Entrypoint to our own wrapper around Vulkan.
// 

// Declare the modules
pub mod errors;
pub mod spec;
pub mod auxillary;
pub mod instance;
pub mod device;
pub mod queue;
pub mod surface;
pub mod swapchain;
pub mod shader;
pub mod descriptors;
pub mod layout;
pub mod render_pass;
pub mod pipeline;
pub mod pools;
pub mod image;
pub mod framebuffer;
pub mod sync;



// Define some useful macros used within this crate
/// Performs a `log`-crate `debug`, but only if that feature is defined
#[cfg(feature = "log")]
macro_rules! debug {
    (target: $target:expr, $($arg:tt)+) => {
        log::debug!($target, $($arg)+)
    };

    ($($arg:tt)+) => {
        log::debug!($($arg)+)
    };
}
#[cfg(not(feature = "log"))]
macro_rules! debug {
    (target: $target:expr, $($arg:tt)+) => { () };

    ($($arg:tt)+) => { () };
}
pub(crate) use debug;

/// Performs a `log`-crate `info`, but only if that feature is defined
#[cfg(feature = "log")]
macro_rules! info {
    (target: $target:expr, $($arg:tt)+) => {
        log::info!($target, $($arg)+)
    };

    ($($arg:tt)+) => {
        log::info!($($arg)+)
    };
}
#[cfg(not(feature = "log"))]
macro_rules! info {
    (target: $target:expr, $($arg:tt)+) => { () };

    ($($arg:tt)+) => { () };
}
pub(crate) use info;

/// Performs a `log`-crate `warn`, but only if that feature is defined
#[cfg(feature = "log")]
macro_rules! _warn {
    (target: $target:expr, $($arg:tt)+) => {
        log::warn!($target, $($arg)+)
    };

    ($($arg:tt)+) => {
        log::warn!($($arg)+)
    };
}
#[cfg(not(feature = "log"))]
macro_rules! _warn {
    (target: $target:expr, $($arg:tt)+) => { () };

    ($($arg:tt)+) => { () };
}
pub(crate) use _warn as warn;

/// Performs a `log`-crate `error`, but only if that feature is defined
#[cfg(feature = "log")]
macro_rules! error {
    (target: $target:expr, $($arg:tt)+) => {
        log::error!($target, $($arg)+)
    };

    ($($arg:tt)+) => {
        log::error!($($arg)+)
    };
}
#[cfg(not(feature = "log"))]
macro_rules! error {
    (target: $target:expr, $($arg:tt)+) => { () };

    ($($arg:tt)+) => { () };
}
pub(crate) use error;



/// Translates a Rust String(-like) to a CString.
macro_rules! to_cstring {
    ($s:expr) => {
        std::ffi::CString::new($s.as_bytes()).unwrap_or_else(|_| panic!("Given string '{}' contains NULL-byte; cannot convert to CString", $s))
    };
}
pub(crate) use to_cstring;
