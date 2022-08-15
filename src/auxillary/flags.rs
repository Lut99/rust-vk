//  FLAGS.rs
//    by Lut99
// 
//  Created:
//    09 Jul 2022, 10:44:36
//  Last edited:
//    15 Aug 2022, 17:55:01
//  Auto updated?
//    Yes
// 
//  Description:
//!   Contains auxillary Flag-structs used as representatives of Vulkan
// 

use std::cmp::PartialEq;
use std::fmt::{Debug, Display};
use std::ops::{BitOr, BitOrAssign};

use ash::vk;


/***** HELPER MACROS *****/
/// Macro that generates the base Flags implementation based on the given Flags values.
macro_rules! flags_new {
    (
        $(#[$doc:ident $($args:tt)*])*
        $name:ident ($type:ident),
        { $(
            $(#[$fdoc:ident $($fargs:tt)*])*
            $fname:ident = $fval:expr $(,)?
        ),+ },
        { $(
            $dmatch:ident => $dresult:literal $(,)?
        ),+ } $(,)?
    ) => {
        $(#[$doc $($args)*])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name($type);

        impl $name {
            $(
                $(#[$fdoc $($fargs)*])*
                pub const $fname: Self = Self($fval)
            );+;


            #[doc = concat!("Constructor for the ", stringify!($name), ", which initializes it with the given raw flags.\n\nNote that this function should not be used with any other values returning from `", stringify!($name), "::as_raw()`.\n\n# Arguments\n- `value`: The raw value to based this ", stringify!($name), " upon.\n\n# Returns\nA new instance of `", stringify!($name), "` with the flags set as in the value.")]
            #[inline]
            pub const fn from_raw(value: $type) -> Self { Self(value) }

            #[doc = concat!("Constructor for the ", stringify!($name), ", which initializes it to empty (no flags set).\n\n# Returns\nA new instance of `", stringify!($name), "` with no flags set.")]
            #[inline]
            pub const fn empty() -> Self { Self(0) }

            #[doc = concat!("Constructor for the ", stringify!($name), ", which initializes it to a full set.\n\n# Returns\nA new instance of `", stringify!($name), "` with all flags set.")]
            #[inline]
            pub const fn all() -> Self { Self(!0) }

            #[doc = concat!("Constructor for the ", stringify!($name), ", which initializes it as a set with the given given Flags combined.\n\n# Returns\nA new instance of `", stringify!($name), "` with all flags set in the union of both given sets.")]
            #[inline]
            pub const fn union(lhs: Self, rhs: Self) -> Self { Self(lhs.0 | rhs.0) }


            #[doc = concat!("Checks if the flags are empty.\n\n#Returns\nReturns true iff this ", stringify!($name), " has no flags set.")]
            #[inline]
            pub const fn is_empty(&self) -> bool { self.0 == 0 }

            #[doc = concat!("Checks if the given ", stringify!($name), " is a subset of this one.\n\n#Arguments\n- `other`: The other set of flags to check.\n\n#Returns\nReturns true iff this ", stringify!($name), " has at least the same bits set as the given one.")]
            #[inline]
            pub const fn check(&self, other: Self) -> bool { (self.0 & other.0) == other.0 }

            #[doc = concat!("Returns the raw integer that we use to represent the set of flags.\n\nNote that this raw number is _not_ guaranteed to be compatible with Vulkan; instead, use the `", stringify!($name), "::from()` function.\n\n#Returns\nThe raw integer carrying the flags.")]
            #[inline]
            pub const fn as_raw(&self) -> $type { self.0 }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                // Construct a list
                let mut first = true;
                let mut i     = 0x1;
                while i != 0 {
                    // Check if this property is enabled
                    if self.0 & i != 0 {
                        // Write the comma if necessary
                        if first { first = false; }
                        else { write!(f, ", ")?; }

                        // Write the name of this property
                        match $name(self.0 & i) {
                            $($name::$dmatch => { write!(f, $dresult)?; }),+
                            value            => { panic!(concat!("Encountered illegal ", stringify!($name), " value '{}'"), value.0); }
                        }
                    }

                    // Increment the i
                    i = i << 1;
                }

                // Done
                Ok(())
            }
        }

        impl BitOr for $name {
            type Output = Self;

            #[inline]
            fn bitor(self, other: Self) -> Self::Output {
                Self(self.0 | other.0)
            }
        }

        impl BitOrAssign for $name {
            #[inline]
            fn bitor_assign(&mut self, other: Self) {
                self.0 |= other.0
            }
        }
    };

    (
        $(#[$doc:ident $($args:tt)*])*
        $name:ident ($type:ident),
        { $(
            $(#[$fdoc:ident $($fargs:tt)*])*
            $fname:ident = $fval:expr $(,)?
        ),+ },
        {} $(,)?
    ) => {
        $(#[$doc $($args)*])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name($type);

        impl $name {
            $(
                $(#[$fdoc $($fargs)*])*
                pub const $fname: Self = Self($fval)
            );+;


            #[doc = concat!("Constructor for the ", stringify!($name), ", which initializes it with the given raw flags.\n\nNote that this function should not be used with any other values returning from `", stringify!($name), "::as_raw()`.\n\n# Arguments\n- `value`: The raw value to based this ", stringify!($name), " upon.\n\n# Returns\nA new instance of `", stringify!($name), "` with the flags set as in the value.")]
            #[inline]
            pub const fn from_raw(value: $type) -> Self { Self(value) }

            #[doc = concat!("Constructor for the ", stringify!($name), " class, which initializes it to empty (no flags set).\n\n# Returns\nA new instance of `", stringify!($name), "` with no flags set.")]
            #[inline]
            pub const fn empty() -> Self { Self(0) }

            #[doc = concat!("Constructor for the ", stringify!($name), " class, which initializes it to a full set.\n\n# Returns\nA new instance of `", stringify!($name), "` with all flags set.")]
            #[inline]
            pub const fn all() -> Self { Self(!0) }

            #[doc = concat!("Constructor for the ", stringify!($name), ", which initializes it as a set with the given given Flags combined.\n\n# Returns\nA new instance of `", stringify!($name), "` with all flags set in the union of both given sets.")]
            #[inline]
            pub const fn union(lhs: Self, rhs: Self) -> Self { Self(lhs.0 | rhs.0) }


            #[doc = concat!("Checks if the given ", stringify!($name), " is a subset of this one.\n\n#Arguments\n- `other`: The other set of flags to check.\n\n#Returns\nReturns true iff this ", stringify!($name), " has at least the same bits set as the given one.")]
            #[inline]
            pub const fn check(&self, other: Self) -> bool { (self.0 & other.0) == other.0 }

            #[doc = concat!("Returns the raw integer that we use to represent the set of flags.\n\nNote that this raw number is _not_ guaranteed to be compatible with Vulkan; instead, use the `", stringify!($name), "::from()` function.\n\n#Returns\nThe raw integer carrying the flags.")]
            #[inline]
            pub const fn as_raw(&self) -> $type { self.0 }
        }

        impl BitOr for $name {
            type Output = Self;

            #[inline]
            fn bitor(self, other: Self) -> Self::Output {
                Self(self.0 | other.0)
            }
        }

        impl BitOrAssign for $name {
            #[inline]
            fn bitor_assign(&mut self, other: Self) {
                self.0 |= other.0
            }
        }
    };

    (
        $(#[$doc:ident $($args:tt)*])*
        $name:ident ($type:ident),
        {},
        { $(
            $dmatch:ident => $dresult:literal $(,)?
        ),+ } $(,)?
    ) => {
        $(#[$doc $($args)*])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name($type);

        impl $name {
            #[doc = concat!("Constructor for the ", stringify!($name), ", which initializes it with the given raw flags.\n\nNote that this function should not be used with any other values returning from `", stringify!($name), "::as_raw()`.\n\n# Arguments\n- `value`: The raw value to based this ", stringify!($name), " upon.\n\n# Returns\nA new instance of `", stringify!($name), "` with the flags set as in the value.")]
            #[inline]
            pub const fn from_raw(value: $type) -> Self { Self(value) }

            #[doc = concat!("Constructor for the ", stringify!($name), ", which initializes it to empty (no flags set).\n\n# Returns\nA new instance of `", stringify!($name), "` with no flags set.")]
            #[inline]
            pub const fn empty() -> Self { Self(0) }

            #[doc = concat!("Constructor for the ", stringify!($name), ", which initializes it to a full set.\n\n# Returns\nA new instance of `", stringify!($name), "` with all flags set.")]
            #[inline]
            pub const fn all() -> Self { Self(!0) }

            #[doc = concat!("Constructor for the ", stringify!($name), ", which initializes it as a set with the given given Flags combined.\n\n# Returns\nA new instance of `", stringify!($name), "` with all flags set in the union of both given sets.")]
            #[inline]
            pub const fn union(lhs: Self, rhs: Self) -> Self { Self(lhs.0 | rhs.0) }


            #[doc = concat!("Checks if the given ", stringify!($name), " is a subset of this one.\n\n#Arguments\n- `other`: The other set of flags to check.\n\n#Returns\nReturns true iff this ", stringify!($name), " has at least the same bits set as the given one.")]
            #[inline]
            pub const fn check(&self, other: Self) -> bool { (self.0 & other.0) == other.0 }

            #[doc = concat!("Returns the raw integer that we use to represent the set of flags.\n\nNote that this raw number is _not_ guaranteed to be compatible with Vulkan; instead, use the `", stringify!($name), "::from()` function.\n\n#Returns\nThe raw integer carrying the flags.")]
            #[inline]
            pub const fn as_raw(&self) -> $type { self.0 }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                // Construct a list
                let mut first = true;
                let mut i     = 0x1;
                while i != 0 {
                    // Check if this property is enabled
                    if self.0 & i != 0 {
                        // Write the comma if necessary
                        if first { first = false; }
                        else { write!(f, ", ")?; }

                        // Write the name of this property
                        match $name(self.0 & i) {
                            $($name::$dmatch => { write!(f, $dresult)?; }),+
                            value            => { panic!(concat!("Encountered illegal ", stringify!($name), " value '{}'"), value.0); }
                        }
                    }

                    // Increment the i
                    i = i << 1;
                }

                // Done
                Ok(())
            }
        }

        impl BitOr for $name {
            type Output = Self;

            #[inline]
            fn bitor(self, other: Self) -> Self::Output {
                Self(self.0 | other.0)
            }
        }

        impl BitOrAssign for $name {
            #[inline]
            fn bitor_assign(&mut self, other: Self) {
                self.0 |= other.0
            }
        }
    };

    (
        $(#[$doc:ident $($args:tt)*])*
        $name:ident ($type:ident),
        {},
        {} $(,)?
    ) => {
        $(#[$doc $($args)*])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name($type);

        impl $name {
            #[doc = concat!("Constructor for the ", stringify!($name), ", which initializes it with the given raw flags.\n\nNote that this function should not be used with any other values returning from `", stringify!($name), "::as_raw()`.\n\n# Arguments\n- `value`: The raw value to based this ", stringify!($name), " upon.\n\n# Returns\nA new instance of `", stringify!($name), "` with the flags set as in the value.")]
            #[inline]
            pub const fn from_raw(value: $type) -> Self { Self(value) }

            #[doc = concat!("Constructor for the ", stringify!($name), " class, which initializes it to empty (no flags set).\n\n# Returns\nA new instance of `", stringify!($name), "` with no flags set.")]
            #[inline]
            pub const fn empty() -> Self { Self(0) }

            #[doc = concat!("Constructor for the ", stringify!($name), " class, which initializes it to a full set.\n\n# Returns\nA new instance of `", stringify!($name), "` with all flags set.")]
            #[inline]
            pub const fn all() -> Self { Self(!0) }

            #[doc = concat!("Constructor for the ", stringify!($name), ", which initializes it as a set with the given given Flags combined.\n\n# Returns\nA new instance of `", stringify!($name), "` with all flags set in the union of both given sets.")]
            #[inline]
            pub const fn union(lhs: Self, rhs: Self) -> Self { Self(lhs.0 | rhs.0) }


            #[doc = concat!("Checks if the given ", stringify!($name), " is a subset of this one.\n\n#Arguments\n- `other`: The other set of flags to check.\n\n#Returns\nReturns true iff this ", stringify!($name), " has at least the same bits set as the given one.")]
            #[inline]
            pub const fn check(&self, other: Self) -> bool { (self.0 & other.0) == other.0 }

            #[doc = concat!("Returns the raw integer that we use to represent the set of flags.\n\nNote that this raw number is _not_ guaranteed to be compatible with Vulkan; instead, use the `", stringify!($name), "::from()` function.\n\n#Returns\nThe raw integer carrying the flags.")]
            #[inline]
            pub const fn as_raw(&self) -> $type { self.0 }
        }

        impl BitOr for $name {
            type Output = Self;

            #[inline]
            fn bitor(self, other: Self) -> Self::Output {
                Self(self.0 | other.0)
            }
        }

        impl BitOrAssign for $name {
            #[inline]
            fn bitor_assign(&mut self, other: Self) {
                self.0 |= other.0
            }
        }
    };
}

/// Macro that generates from-function for a flag that derives from a Vulkan value.
macro_rules! flags_from {
    (vk::$from:ident, $to:ident, $($match:ident => $target:ident $(,)?),+) => {
        flags_from!(vk::$from, $to, $(vk::$from::$match => $to::$target),+);
    };

    (vk::$from:ident, $to:ident, $($match:path => $target:ident $(,)?),+) => {
        flags_from!(vk::$from, $to, $($match => $to::$target),+);
    };

    (vk::$from:ident, $to:ident, $($match:ident => $target:path $(,)?),+) => {
        flags_from!(vk::$from, $to, $(vk::$from::$match => $target),+);
    };

    (vk::$from:ident, $to:ident, $($match:path => $target:path $(,)?),+) => {
        impl From<vk::$from> for $to {
            fn from(value: vk::$from) -> $to {
                // Construct the resulting flag iteratively
                let mut result: $to = $to::empty();
                $(if (value & $match).as_raw() != 0 { result |= $target });+
                result
            }
        }

        impl From<&vk::$from> for $to {
            fn from(value: &vk::$from) -> $to {
                // Construct the resulting flag iteratively
                let mut result: $to = $to::empty();
                $(if ((*value) & $match).as_raw() != 0 { result |= $target });+
                result
            }
        }

        impl From<$to> for vk::$from {
            fn from(value: $to) -> vk::$from {
                // Construct the resulting flag iteratively
                let mut result: vk::$from = vk::$from::empty();
                $(if value.check($target) { result |= $match });+
                result
            }
        }

        impl From<&$to> for vk::$from {
            fn from(value: &$to) -> vk::$from {
                // Construct the resulting flag iteratively
                let mut result: vk::$from = vk::$from::empty();
                $(if value.check($target) { result |= $match });+
                result
            }
        }
    };
}

/// Macro that generates both a base 'single-flag' struct and a 'collection-of-flags' struct.
macro_rules! flags_single_new {
    (
        $(#[$doc:ident $($args:tt)*])*
        $name:ident ($type:ident), $name_flags:ident,
        { $(
            $(#[$fdoc:ident $($fargs:tt)*])*
            $fname:ident = $fval:expr $(,)?
        ),+ },
        { $(
            $dmatch:ident => $dresult:literal $(,)?
        ),+ } $(,)?
    ) => {
        $(#[$doc $($args)*])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name($type);

        impl $name {
            $(
                $(#[$fdoc $($fargs)*])*
                pub const $fname: Self = Self($fval)
            );+;


            #[doc = concat!("Constructor for the ", stringify!($name), ", which initializes it with the given raw flags.\n\nNote that this function should not be used with any other values returning from `", stringify!($name), "::as_raw()`.\n\n# Arguments\n- `value`: The raw value to based this ", stringify!($name), " upon.\n\n# Returns\nA new instance of `", stringify!($name), "` with the flags set as in the value.")]
            #[inline]
            pub const fn from_raw(value: $type) -> Self { Self(value) }

            #[doc = concat!("Returns the raw integer that we use to represent the set of flags.\n\nNote that this raw number is _not_ guaranteed to be compatible with Vulkan; instead, use the `", stringify!($name), "::from()` function.\n\n#Returns\nThe raw integer carrying the flags.")]
            #[inline]
            pub const fn as_raw(&self) -> $type { self.0 }
        }

        impl Display for $name {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match *self {
                    $($name::$dmatch => { write!(f, $dresult) }),+
                    _                => { write!(f, "???") }
                }
            }
        }



        flags_new!(
            $name_flags($type),
            { $(
                #[doc = concat!("Defines a set of ", stringify!($name), " flags.")]
                $fname = $fval
            ),+ },
            { $(
                $dmatch => $dresult
            ),+ },
        );

        impl From<$name> for $name_flags {
            #[inline]
            fn from(value: $name) -> $name_flags {
                $name_flags(value.0)
            }
        }
    };

    (
        $(#[$doc:ident $($args:tt)*])*
        $name:ident ($type:ident), $name_flags:ident,
        { $(
            $(#[$fdoc:ident $($fargs:tt)*])*
            $fname:ident = $fval:expr $(,)?
        ),+ },
        {} $(,)?
    ) => {
        $(#[$doc $($args)*])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name($type);

        impl $name {
            $(
                $(#[$fdoc $($fargs)*])*
                pub const $fname: Self = Self($fval)
            );+;


            #[doc = concat!("Constructor for the ", stringify!($name), ", which initializes it with the given raw flags.\n\nNote that this function should not be used with any other values returning from `", stringify!($name), "::as_raw()`.\n\n# Arguments\n- `value`: The raw value to based this ", stringify!($name), " upon.\n\n# Returns\nA new instance of `", stringify!($name), "` with the flags set as in the value.")]
            #[inline]
            pub const fn from_raw(value: $type) -> Self { Self(value) }

            #[doc = concat!("Returns the raw integer that we use to represent the set of flags.\n\nNote that this raw number is _not_ guaranteed to be compatible with Vulkan; instead, use the `", stringify!($name), "::from()` function.\n\n#Returns\nThe raw integer carrying the flags.")]
            #[inline]
            pub const fn as_raw(&self) -> $type { self.0 }
        }



        flags_new!(
            $name_flags($type),
            { $(
                #[doc = concat!("Defines a set of ", stringify!($name), " flags.")]
                $fname = $fval
            ),+ },
            { $(
                $dmatch => $dresult
            ),+ },
        );

        impl From<$name> for $name_flags {
            #[inline]
            fn from(value: $name) -> $name_flags {
                $name_flags(value.0)
            }
        }
    };

    (
        $(#[$doc:ident $($args:tt)*])*
        $name:ident ($type:ident), $name_flags:ident,
        {},
        { $(
            $dmatch:ident => $dresult:literal $(,)?
        ),+ } $(,)?
    ) => {
        $(#[$doc $($args)*])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name($type);

        impl $name {
            #[doc = concat!("Constructor for the ", stringify!($name), ", which initializes it with the given raw flags.\n\nNote that this function should not be used with any other values returning from `", stringify!($name), "::as_raw()`.\n\n# Arguments\n- `value`: The raw value to based this ", stringify!($name), " upon.\n\n# Returns\nA new instance of `", stringify!($name), "` with the flags set as in the value.")]
            #[inline]
            pub const fn from_raw(value: $type) -> Self { Self(value) }

            #[doc = concat!("Returns the raw integer that we use to represent the set of flags.\n\nNote that this raw number is _not_ guaranteed to be compatible with Vulkan; instead, use the `", stringify!($name), "::from()` function.\n\n#Returns\nThe raw integer carrying the flags.")]
            #[inline]
            pub const fn as_raw(&self) -> $type { self.0 }
        }

        impl Display for $name {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $($name::$dmatch => { write!(f, $dresult)?; }),+,
                    value            => { write!(f, "???")?; }
                }
            }
        }



        flags_new!(
            $name_flags($type),
            {},
            { $(
                $dmatch => $dresult
            ),+ },
        );

        impl From<$name> for $name_flags {
            #[inline]
            fn from(value: $name) -> $name_flags {
                $name_flags(value.0)
            }
        }
    };

    (
        $(#[$doc:ident $($args:tt)*])*
        $name:ident ($type:ident), $name_flags:ident,
        {},
        {} $(,)?
    ) => {
        $(#[$doc $($args)*])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name($type);

        impl $name {
            #[doc = concat!("Constructor for the ", stringify!($name), ", which initializes it with the given raw flags.\n\nNote that this function should not be used with any other values returning from `", stringify!($name), "::as_raw()`.\n\n# Arguments\n- `value`: The raw value to based this ", stringify!($name), " upon.\n\n# Returns\nA new instance of `", stringify!($name), "` with the flags set as in the value.")]
            #[inline]
            pub const fn from_raw(value: $type) -> Self { Self(value) }

            #[doc = concat!("Returns the raw integer that we use to represent the set of flags.\n\nNote that this raw number is _not_ guaranteed to be compatible with Vulkan; instead, use the `", stringify!($name), "::from()` function.\n\n#Returns\nThe raw integer carrying the flags.")]
            #[inline]
            pub const fn as_raw(&self) -> $type { self.0 }
        }



        flags_new!(
            $name_flags($type),
            {},
            {},
        );

        impl From<$name> for $name_flags {
            #[inline]
            fn from(value: $name) -> $name_flags {
                $name_flags(value.0)
            }
        }
    };
}

/// Macro that generates from-function for both a base 'single-flag' struct and a 'collection-of-flags' struct that derives from a Vulkan value.
macro_rules! flags_single_from {
    (vk::$from:ident, $to:ident,$to_flags:ident,  $($match:ident => $target:ident $(,)?),+) => {
        flags_single_from!(vk::$from, $to, $to_flags, $(vk::$from::$match => $target),+);
    };

    (vk::$from:ident, $to:ident, $to_flags:ident, $($match:path => $target:ident $(,)?),+) => {
        impl From<vk::$from> for $to {
            #[inline]
            fn from(value: vk::$from) -> $to {
                match value {
                    $($match => $to::$target),+,
                    value               => { panic!(concat!("Encountered illegal value '{}' for ", stringify!(vk::$from)), value.as_raw()); }
                }
            }
        }

        impl From<&vk::$from> for $to {
            #[inline]
            fn from(value: &vk::$from) -> $to {
                match *value {
                    $($match => $to::$target),+,
                    value               => { panic!(concat!("Encountered illegal value '{}' for ", stringify!(vk::$from)), value.as_raw()); }
                }
            }
        }

        impl From<$to> for vk::$from {
            #[inline]
            fn from(value: $to) -> vk::$from {
                match value {
                    $($to::$target => $match),+,
                    value          => { panic!(concat!("Encountered illegal value '{}' for ", stringify!($to)), value.0); }
                }
            }
        }

        impl From<&$to> for vk::$from {
            #[inline]
            fn from(value: &$to) -> vk::$from {
                match *value {
                    $($to::$target => $match),+,
                    value          => { panic!(concat!("Encountered illegal value '{}' for ", stringify!($to)), value.0); }
                }
            }
        }



        impl From<vk::$from> for $to_flags {
            fn from(value: vk::$from) -> $to_flags {
                // Construct the resulting flag iteratively
                let mut result: $to_flags = $to_flags::empty();
                $(if (value & $match).as_raw() != 0 { result |= $to_flags::$target });+
                result
            }
        }

        impl From<&vk::$from> for $to_flags {
            fn from(value: &vk::$from) -> $to_flags {
                // Construct the resulting flag iteratively
                let mut result: $to_flags = $to_flags::empty();
                $(if (*value & $match).as_raw() != 0 { result |= $to_flags::$target });+
                result
            }
        }

        impl From<$to_flags> for vk::$from {
            fn from(value: $to_flags) -> vk::$from {
                // Construct the resulting flag iteratively
                let mut result: vk::$from = vk::$from::empty();
                $(if value.check($to_flags::$target) { result |= $match });+
                result
            }
        }

        impl From<&$to_flags> for vk::$from {
            fn from(value: &$to_flags) -> vk::$from {
                // Construct the resulting flag iteratively
                let mut result: vk::$from = vk::$from::empty();
                $(if value.check($to_flags::$target) { result |= $match });+
                result
            }
        }
    };
}

/// Macro that generates from-function for both a base 'single-flag' struct and a 'collection-of-flags' struct that derives from a primitive 'raw' value.
macro_rules! flags_single_from_raw {
    ($from:ident, $to:ident($type:ident), $to_flags:ident) => {
        impl From<$from> for $to {
            #[inline]
            fn from(value: $from) -> $to {
                Self(value as $type)
            }
        }

        impl From<$to> for $from {
            #[inline]
            fn from(value: $to) -> $from {
                value.0 as $from
            }
        }



        impl From<$from> for $to_flags {
            #[inline]
            fn from(value: $from) -> $to_flags {
                Self(value as $type)
            }
        }

        impl From<$to_flags> for $from {
            fn from(value: $to_flags) -> $from {
                value.0 as $from
            }
        }
    };
}





/***** DEVICES *****/
flags_new!(
    /// Contains information about what a device heap supports, exactly.
    HeapPropertyFlags(u8),
    {
        /// The heap corresponds to device-local memory.
        DEVICE_LOCAL   = 0x01,
        /// In the case of a multi-instance logical device, this heap has a per-device instance. That means that (by default) every allocation will be replicated to each heap.
        MULTI_INSTANCE = 0x02,
    },
    {
        DEVICE_LOCAL   => "DEVICE_LOCAL",
        MULTI_INSTANCE => "MULTI_INSTANCE",
    },
);

flags_from!(vk::MemoryHeapFlags, HeapPropertyFlags, 
    DEVICE_LOCAL       => DEVICE_LOCAL,
    MULTI_INSTANCE     => MULTI_INSTANCE,
    MULTI_INSTANCE_KHR => MULTI_INSTANCE,
);





/***** SHADERS *****/
flags_single_new!(
    /// The ShaderStage enumerates possible stages where shaders live.
    ShaderStage(u16), ShaderStageFlags,
    {
        /// The Vertex stage
        VERTEX                  = 0x0001,
        /// The control stage of the Tesselation stage
        TESSELLATION_CONTROL    = 0x0002,
        /// The evaluation stage of the Tesselation stage
        TESSELLATION_EVALUATION = 0x0004,
        /// The Geometry stage
        GEOMETRY                = 0x0008,
        /// The Fragment stage
        FRAGMENT                = 0x0010,
        /// The Compute stage
        COMPUTE                 = 0x0020,
    },
    {
        VERTEX                  => "Vertex",
        TESSELLATION_CONTROL    => "Tesselation (control)",
        TESSELLATION_EVALUATION => "Tesselation (evaluation)",
        GEOMETRY                => "Geometry",
        FRAGMENT                => "Fragment",
        COMPUTE                 => "Compute",
    },
);

flags_single_from!(vk::ShaderStageFlags, ShaderStage, ShaderStageFlags,
    VERTEX                  => VERTEX,
    TESSELLATION_CONTROL    => TESSELLATION_CONTROL,
    TESSELLATION_EVALUATION => TESSELLATION_EVALUATION,
    GEOMETRY                => GEOMETRY,
    FRAGMENT                => FRAGMENT,
    COMPUTE                 => COMPUTE,
);





/***** RENDER PASSES *****/
flags_new!(
    /// Defines kinds of operations that are relevant for synchronization.
    AccessFlags(u32),
    {
        /// Defines an operation that reads during the DRAW_INDIRECT pipeline stage(?)
        INDIRECT_COMMAND_READ   = 0x00001,
        /// Defines a read operation in the index buffer.
        INDEX_READ              = 0x00002,
        /// Defines a read operation of a vertex attribute in the vertex buffer.
        VERTEX_ATTRIBUTE_READ   = 0x00004,
        /// Defines a read operation of a uniform buffer.
        UNIFORM_READ            = 0x00008,
        /// Defines a read operation of an input attachment.
        INPUT_ATTACHMENT_READ   = 0x00010,
        /// Defines a read operation in a shader.
        SHADER_READ             = 0x00020,
        /// Defines a write operation in a shader.
        SHADER_WRITE            = 0x00040,
        /// Defines a read operation from a colour attachment.
        COLOUR_ATTACHMENT_READ  = 0x00080,
        /// Defines a write operation from a colour attachment.
        COLOUR_ATTACHMENT_WRITE = 0x00100,
        /// Defines a read operation from a depth stencil.
        DEPTH_STENCIL_READ      = 0x00200,
        /// Defines a write operation from a depth stencil.
        DEPTH_STENCIL_WRITE     = 0x00400,
        /// Defines a read operation during the transferring of buffers or images.
        TRANSFER_READ           = 0x00800,
        /// Defines a write operation during the transferring of buffers or images.
        TRANSFER_WRITE          = 0x01000,
        /// Defines a read operation performed by the host (I assume on GPU resources in shared memory).
        HOST_READ               = 0x02000,
        /// Defines a write operation performed by the host (I assume on GPU resources in shared memory).
        HOST_WRITE              = 0x04000,
        /// Defines _any_ read operation.
        MEMORY_READ             = 0x08000,
        /// Defines _any_ write operation.
        MEMORY_WRITE            = 0x10000,
    },
    {
        INDIRECT_COMMAND_READ   => "INDIRECT_COMMAND_READ",
        INDEX_READ              => "INDEX_READ",
        VERTEX_ATTRIBUTE_READ   => "VERTEX_ATTRIBUTE_READ",
        UNIFORM_READ            => "UNIFORM_READ",
        INPUT_ATTACHMENT_READ   => "INPUT_ATTACHMENT_READ",
        SHADER_READ             => "SHADER_READ",
        SHADER_WRITE            => "SHADER_WRITE",
        COLOUR_ATTACHMENT_READ  => "COLOUR_ATTACHMENT_READ",
        COLOUR_ATTACHMENT_WRITE => "COLOUR_ATTACHMENT_WRITE",
        TRANSFER_READ           => "TRANSFER_READ",
        TRANSFER_WRITE          => "TRANSFER_WRITE",
        HOST_READ               => "HOST_READ",
        HOST_WRITE              => "HOST_WRITE",
        MEMORY_READ             => "MEMORY_READ",
        MEMORY_WRITE            => "MEMORY_WRITE",
    },
);

flags_from!(vk::AccessFlags, AccessFlags,
    INDIRECT_COMMAND_READ  => INDIRECT_COMMAND_READ,
    INDEX_READ             => INDEX_READ,
    VERTEX_ATTRIBUTE_READ  => VERTEX_ATTRIBUTE_READ,
    UNIFORM_READ           => UNIFORM_READ,
    INPUT_ATTACHMENT_READ  => INPUT_ATTACHMENT_READ,
    SHADER_READ            => SHADER_READ,
    SHADER_WRITE           => SHADER_WRITE,
    COLOR_ATTACHMENT_READ  => COLOUR_ATTACHMENT_READ,
    COLOR_ATTACHMENT_WRITE => COLOUR_ATTACHMENT_WRITE,
    TRANSFER_READ          => TRANSFER_READ,
    TRANSFER_WRITE         => TRANSFER_WRITE,
    HOST_READ              => HOST_READ,
    HOST_WRITE             => HOST_WRITE,
    MEMORY_READ            => MEMORY_READ,
    MEMORY_WRITE           => MEMORY_WRITE,
);



flags_new!(
    /// Defines the kind of dependency that we're defining.
    DependencyFlags(u8),
    {
        /// The dependency is local to each framebuffer (must be given if the stages include framebuffers).
        FRAMEBUFFER_LOCAL = 0x01,
        /// Every subpass has more than one ImageView that needs dependencies (must be given if so).
        VIEW_LOCAL        = 0x02,
        /// If the dependency is not local to a device, this flag should be given.
        NOT_DEVICE_LOCAL  = 0x04,
    },
    {
        FRAMEBUFFER_LOCAL => "FRAMEBUFFER_LOCAL",
        VIEW_LOCAL        => "VIEW_LOCAL",
        NOT_DEVICE_LOCAL  => "NOT_DEVICE_LOCAL",
    },
);

flags_from!(vk::DependencyFlags, DependencyFlags,
    vk::DependencyFlags::BY_REGION    => DependencyFlags::FRAMEBUFFER_LOCAL,
    vk::DependencyFlags::VIEW_LOCAL   => DependencyFlags::VIEW_LOCAL,
    vk::DependencyFlags::DEVICE_GROUP => DependencyFlags::NOT_DEVICE_LOCAL,
);



flags_single_new!(
    /// The Pipeline stage where a shader or a resource lives.
    PipelineStage(u32), PipelineStageFlags,
    {
        /// Defines the stage before anything of the pipeline is run.
        TOP_OF_PIPE                    = 0x00001,
        /// The indirect draw stage.
        DRAW_INDIRECT                  = 0x00002,
        /// The stage where vertices (and indices) are read.
        VERTEX_INPUT                   = 0x00004,
        /// The Vertex shader stage.
        VERTEX_SHADER                  = 0x00008,
        /// The control stage of the Tesselation shader stage.
        TESSELLATION_CONTROL_SHADER    = 0x00010,
        /// The evaluation stage of the Tesselation shader stage.
        TESSELLATION_EVALUATION_SHADER = 0x00020,
        /// The Geometry shader stage.
        GEOMETRY_SHADER                = 0x00040,
        /// The Fragment shader stage.
        FRAGMENT_SHADER                = 0x00080,
        /// The stage where early fragments tests (depth and stencil tests before fragment shading) are performed. This stage also performs subpass load operations for framebuffers with depth attachments.
        EARLY_FRAGMENT_TESTS           = 0x00100,
        /// The stage where late fragments tests (depth and stencil tests after fragment shading) are performed. This stage also performs subpass write operations for framebuffers with depth attachments.
        LATE_FRAGMENT_TESTS            = 0x00200,
        /// The stage where the fragments are written to the colour attachment (after blending).
        COLOUR_ATTACHMENT_OUTPUT       = 0x00400,
        /// The stage where any compute shaders may be processed.
        COMPUTE_SHADER                 = 0x00800,
        /// The stage where any data is transferred to and from buffers and images (all copy commands, blit, resolve and clear commands (except vkCmdClearAttachments).
        TRANSFER                       = 0x01000,
        /// Defines the stage after the entire pipeline has been completed.
        BOTTOM_OF_PIPE                 = 0x02000,
        /// A (pseudo-)stage where host access to a device is performed.
        HOST                           = 0x04000,
        /// Collection for all graphics-related stages.
        ALL_GRAPHICS                   = 0x08000,
        /// Collection for all commandbuffer-invoked stages _supported on the executing queue_.
        ALL_COMMANDS                   = 0x10000,
    },
    {
        TOP_OF_PIPE                    => "TOP_OF_PIPE",
        DRAW_INDIRECT                  => "DRAW_INDIRECT",
        VERTEX_INPUT                   => "VERTEX_INPUT",
        VERTEX_SHADER                  => "VERTEX_SHADER",
        TESSELLATION_CONTROL_SHADER    => "TESSELLATION_CONTROL_SHADER",
        TESSELLATION_EVALUATION_SHADER => "TESSELLATION_EVALUATION_SHADER",
        GEOMETRY_SHADER                => "GEOMETRY_SHADER",
        FRAGMENT_SHADER                => "FRAGMENT_SHADER",
        EARLY_FRAGMENT_TESTS           => "EARLY_FRAGMENT_TESTS",
        LATE_FRAGMENT_TESTS            => "LATE_FRAGMENT_TESTS",
        COLOUR_ATTACHMENT_OUTPUT       => "COLOUR_ATTACHMENT_OUTPUT",
        COMPUTE_SHADER                 => "COMPUTE_SHADER",
        TRANSFER                       => "TRANSFER",
        BOTTOM_OF_PIPE                 => "BOTTOM_OF_PIPE",
        HOST                           => "HOST",
        ALL_GRAPHICS                   => "ALL_GRAPHICS",
        ALL_COMMANDS                   => "ALL_COMMANDS",
    },
);

flags_single_from!(vk::PipelineStageFlags, PipelineStage, PipelineStageFlags,
    vk::PipelineStageFlags::TOP_OF_PIPE                    => TOP_OF_PIPE,
    vk::PipelineStageFlags::DRAW_INDIRECT                  => DRAW_INDIRECT,
    vk::PipelineStageFlags::VERTEX_INPUT                   => VERTEX_INPUT,
    vk::PipelineStageFlags::VERTEX_SHADER                  => VERTEX_SHADER,
    vk::PipelineStageFlags::TESSELLATION_CONTROL_SHADER    => TESSELLATION_CONTROL_SHADER,
    vk::PipelineStageFlags::TESSELLATION_EVALUATION_SHADER => TESSELLATION_EVALUATION_SHADER,
    vk::PipelineStageFlags::GEOMETRY_SHADER                => GEOMETRY_SHADER,
    vk::PipelineStageFlags::FRAGMENT_SHADER                => FRAGMENT_SHADER,
    vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS           => EARLY_FRAGMENT_TESTS,
    vk::PipelineStageFlags::LATE_FRAGMENT_TESTS            => LATE_FRAGMENT_TESTS,
    vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT        => COLOUR_ATTACHMENT_OUTPUT,
    vk::PipelineStageFlags::COMPUTE_SHADER                 => COMPUTE_SHADER,
    vk::PipelineStageFlags::TRANSFER                       => TRANSFER,
    vk::PipelineStageFlags::BOTTOM_OF_PIPE                 => BOTTOM_OF_PIPE,
    vk::PipelineStageFlags::HOST                           => HOST,
    vk::PipelineStageFlags::ALL_GRAPHICS                   => ALL_GRAPHICS,
    vk::PipelineStageFlags::ALL_COMMANDS                   => ALL_COMMANDS,
);





/***** PIPELINES *****/
flags_new!(
    /// Defines the channel mask to use when writing.
    ColourComponentFlags(u8),
    {
        /// A colour mask for only the red colour channel.
        RED   = 0b00000001,
        /// A colour mask for only the green colour channel.
        GREEN = 0b00000010,
        /// A colour mask for only the blue colour channel.
        BLUE  = 0b00000100,
        /// A colour mask for only the alpha channel.
        ALPHA = 0b00001000,
    },
    {
        RED   => "Red",
        GREEN => "Green",
        BLUE  => "Blue",
        ALPHA => "Alpha",
    },
);

flags_from!(vk::ColorComponentFlags, ColourComponentFlags,
    vk::ColorComponentFlags::R => ColourComponentFlags::RED,
    vk::ColorComponentFlags::G => ColourComponentFlags::GREEN,
    vk::ColorComponentFlags::B => ColourComponentFlags::BLUE,
    vk::ColorComponentFlags::A => ColourComponentFlags::ALPHA,
);





/***** MEMORY POOLS *****/
flags_new!(
    /// Lists properties of certain memory areas.
    MemoryPropertyFlags(u16),
    {
        /// Memory should be local to the Device (i.e., not some shared memory pool).
        DEVICE_LOCAL     = 0x0001,
        /// Memory should be writeable/readable by the Host.
        HOST_VISIBLE     = 0x0002,
        /// Memory should be coherent with the host (not requiring separate flush calls).
        HOST_COHERENT    = 0x0004,
        /// Memory is cached, which is faster but non-coherent.
        HOST_CACHED      = 0x0008,
        /// Memory might need to be allocated on first access.
        LAZILY_ALLOCATED = 0x0010,
        /// Memory is protected; only Device may access it and some special queue operations.
        PROTECTED        = 0x0020,
    },
    {
        DEVICE_LOCAL     => "DEVICE_LOCAL",
        HOST_VISIBLE     => "HOST_VISIBLE",
        HOST_COHERENT    => "HOST_COHERENT",
        HOST_CACHED      => "HOST_CACHED",
        LAZILY_ALLOCATED => "LAZILY_ALLOCATED",
        PROTECTED        => "PROTECTED",
    },
);

flags_from!(vk::MemoryPropertyFlags, MemoryPropertyFlags, 
    vk::MemoryPropertyFlags::DEVICE_LOCAL     => MemoryPropertyFlags::DEVICE_LOCAL,
    vk::MemoryPropertyFlags::HOST_VISIBLE     => MemoryPropertyFlags::HOST_VISIBLE,
    vk::MemoryPropertyFlags::HOST_COHERENT    => MemoryPropertyFlags::HOST_COHERENT,
    vk::MemoryPropertyFlags::HOST_CACHED      => MemoryPropertyFlags::HOST_CACHED,
    vk::MemoryPropertyFlags::LAZILY_ALLOCATED => MemoryPropertyFlags::LAZILY_ALLOCATED,
    vk::MemoryPropertyFlags::PROTECTED        => MemoryPropertyFlags::PROTECTED,
);





/***** COMMANDS POOLS *****/
flags_new!(
    /// Flags for the CommandPool construction.
    #[derive(Hash)]
    CommandBufferFlags(u8),
    {
        /// The buffers coming from this CommandPool will be short-lived.
        TRANSIENT   = 0x01,
        /// The buffers coming from this CommandPool may be individually reset instead of only all at once by resetting the pool.
        ALLOW_RESET = 0x02,
    },
    {
        TRANSIENT   => "TRANSIENT",
        ALLOW_RESET => "ALLOW_RESET",
    }
);

flags_from!(vk::CommandPoolCreateFlags, CommandBufferFlags,
    vk::CommandPoolCreateFlags::TRANSIENT            => CommandBufferFlags::TRANSIENT,
    vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER => CommandBufferFlags::ALLOW_RESET,
);



flags_new!(
    /// Flags to set options when beginning a command buffer.
    CommandBufferUsageFlags(u8),
    {
        /// Tells the Vulkan driver that this command buffer will only be submitted once, and reset or destroyed afterwards.
        ONE_TIME_SUBMIT  = 0x01,
        /// If the CommandBuffer is secondary, then this bit indicates that it lives entirely within the RenderPass.
        RENDER_PASS_ONLY = 0x02,
        /// The buffer can be resubmitted while it is pending and recorded into multiple primary command buffers.
        SIMULTANEOUS_USE = 0x04,
    },
    {
        ONE_TIME_SUBMIT  => "ONE_TIME_SUBMIT",
        RENDER_PASS_ONLY => "RENDER_PASS_ONLY",
        SIMULTANEOUS_USE => "SIMULTANEOUS_USE",
    }
);

flags_from!(vk::CommandBufferUsageFlags, CommandBufferUsageFlags,
    vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT      => CommandBufferUsageFlags::ONE_TIME_SUBMIT,
    vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE => CommandBufferUsageFlags::RENDER_PASS_ONLY,
    vk::CommandBufferUsageFlags::SIMULTANEOUS_USE     => CommandBufferUsageFlags::SIMULTANEOUS_USE,
);





/***** MEMORY POOLS *****/
flags_single_new!(
    /// Define a single type of memory that a device has to offer.
    /// 
    /// Note: because the actual list is device-dependent, there are no constants available for this "enum" implementation.
    DeviceMemoryType(u32), DeviceMemoryTypeFlags,
    {},
    {},
);

impl Display for DeviceMemoryType {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for DeviceMemoryTypeFlags {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

flags_single_from_raw!(u32, DeviceMemoryType(u32), DeviceMemoryTypeFlags);
flags_single_from_raw!(usize, DeviceMemoryType(u32), DeviceMemoryTypeFlags);



flags_new!(
    /// The BufferUsageFlags that determine what we can use a buffer for.
    BufferUsageFlags(u16),
    {
        /// The buffer may be used as a source buffer in a memory transfer operation.
        TRANSFER_SRC = 0x0001,
        /// The buffer may be used as a target buffer in a memory transfer operation.
        TRANSFER_DST = 0x0002,
        /// The buffer may be used as a uniform texel buffer.
        /// 
        /// Uniform buffers are much smaller but slightly faster than storage buffers.
        UNIFORM_TEXEL_BUFFER = 0x0004,
        /// The buffer may be used as a storage texel buffer.
        /// 
        /// Storage buffers are much larger but slightly slower than uniform buffers.
        STORAGE_TEXEL_BUFFER = 0x0008,
        /// The buffer may be used as a uniform buffer.
        /// 
        /// Uniform buffers are much smaller but slightly faster than storage buffers.
        UNIFORM_BUFFER = 0x0010,
        /// The buffer may be used as a storage buffer.
        /// 
        /// Storage buffers are much larger but slightly slower than uniform buffers.
        STORAGE_BUFFER = 0x0020,
        /// The buffer may be used to storage indices.
        INDEX_BUFFER = 0x0040,
        /// The buffer may be used to storage vertices.
        VERTEX_BUFFER = 0x0080,
        /// The buffer may be used for indirect draw commands (various applications).
        INDIRECT_BUFFER = 0x0100,
    },
    {
        TRANSFER_SRC         => "Transfer (source)",
        TRANSFER_DST         => "Transfer (destination)",
        UNIFORM_TEXEL_BUFFER => "Uniform texel buffer",
        STORAGE_TEXEL_BUFFER => "Storage texel buffer",
        UNIFORM_BUFFER       => "Uniform buffer",
        STORAGE_BUFFER       => "Storage buffer",
        INDEX_BUFFER         => "Index buffer",
        VERTEX_BUFFER        => "Vertex buffer",
        INDIRECT_BUFFER      => "Indirect buffer",
    },
);

flags_from!(vk::BufferUsageFlags, BufferUsageFlags,
    vk::BufferUsageFlags::TRANSFER_SRC         => BufferUsageFlags::TRANSFER_SRC,
    vk::BufferUsageFlags::TRANSFER_DST         => BufferUsageFlags::TRANSFER_DST,
    vk::BufferUsageFlags::UNIFORM_TEXEL_BUFFER => BufferUsageFlags::UNIFORM_TEXEL_BUFFER,
    vk::BufferUsageFlags::STORAGE_TEXEL_BUFFER => BufferUsageFlags::STORAGE_TEXEL_BUFFER,
    vk::BufferUsageFlags::UNIFORM_BUFFER       => BufferUsageFlags::UNIFORM_BUFFER,
    vk::BufferUsageFlags::STORAGE_BUFFER       => BufferUsageFlags::STORAGE_BUFFER,
    vk::BufferUsageFlags::INDEX_BUFFER         => BufferUsageFlags::INDEX_BUFFER,
    vk::BufferUsageFlags::VERTEX_BUFFER        => BufferUsageFlags::VERTEX_BUFFER,
    vk::BufferUsageFlags::INDIRECT_BUFFER      => BufferUsageFlags::INDIRECT_BUFFER,
);





/***** IMAGES *****/
flags_single_new!(
    /// Defines the number of samples to multi-sample.
    SampleCount(u8), SampleCountFlags,
    {
        /// Only one sample
        ONE        = 0x01,
        /// Take two samples
        TWO        = 0x02,
        /// Take four samples
        FOUR       = 0x04,
        /// Take eight samples
        EIGHT      = 0x08,
        /// Now we're getting somewhere: sixteen samples
        SIXTEEN    = 0x10,
        /// _Hardcore_: thirty-two samples!
        THIRTY_TWO = 0x20,
        /// What?! Sixty-four whole samples?! :O
        SIXTY_FOUR = 0x40,
    },
    {
        ONE        => "1",
        TWO        => "2",
        FOUR       => "4",
        EIGHT      => "8",
        SIXTEEN    => "16",
        THIRTY_TWO => "32",
        SIXTY_FOUR => "64",
    },
);

flags_single_from!(vk::SampleCountFlags, SampleCount, SampleCountFlags,
    vk::SampleCountFlags::TYPE_1  => ONE,
    vk::SampleCountFlags::TYPE_2  => TWO,
    vk::SampleCountFlags::TYPE_4  => FOUR,
    vk::SampleCountFlags::TYPE_8  => EIGHT,
    vk::SampleCountFlags::TYPE_16 => SIXTEEN,
    vk::SampleCountFlags::TYPE_32 => THIRTY_TWO,
    vk::SampleCountFlags::TYPE_64 => SIXTY_FOUR,
);
