//  MOD.rs
//    by Lut99
// 
//  Created:
//    09 Jul 2022, 12:20:31
//  Last edited:
//    06 Aug 2022, 11:01:05
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements auxillary structs, enums & flags (special kind of
//!   structs)
// 

/// The module containing enums.
pub mod enums;
/// The module containing flags.
pub mod flags;
/// The module containing (parameter) structs.
pub mod structs;


/***** MACROS *****/
/// Exports the pointer of a vector or NULL if that vector is empty.
#[macro_export]
macro_rules! vec_as_ptr {
    ($vec:ident) => {
        (if $vec.is_empty() { ptr::null() } else { $vec.as_ptr() })
    };
}

/// Prints a default destroy message for 'self'
#[macro_export]
macro_rules! log_destroy {
    ($self:ident,$type:path) => {
        $crate::debug!(concat!("Destroying ", stringify!($type), " {:?}..."), $self as *const $type)
    };

    ($self:ident,$type:path,$name:expr) => {
        $crate::debug!(concat!("Destroying ", stringify!($type), " '{}' ({:?})..."), $name, $self as *const $type)
    }
}
