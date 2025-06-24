#![cfg_attr(not(test), no_std)]
#![allow(rustdoc::redundant_explicit_links)]
#![warn(clippy::undocumented_unsafe_blocks)]

//! Provides stable const implementations for some things missing from the standard library.
//!
//! Currently implemented are
//! - Functions in [`result`](crate::result) to unwrap [`Result`](core::result::Result)s with generics or drop glue
//! - Functions in the [`concat`](crate::concat) module to concat const strings and byte slices.
//! - [`destruct_tuple`](crate::destruct_tuple) to destructure tuples with generic types or types with drop glue in them
//! - [`nonnull_from`](crate::mem::nonnull_from) to create [`NonNull`](core::ptr::NonNull)s from mutable and regular references
//!   conveniently
//! - [`man_drop_ref`](crate::mem::man_drop_ref)/[`man_drop_mut`](crate::mem::man_drop_mut) as a workaround for the lack of const
//!   [`Deref`](core::ops::Deref) implementations

pub extern crate type_const;
pub use type_const::{value_of, Const};

pub mod concat;
pub mod mem;
pub mod result;
pub mod slice;

#[doc(hidden)]
#[macro_export]
macro_rules! __infer {
    ($($_:tt)*) => {
        _
    };
}

/// Allows destructuring tuples in `const` contexts, regardless of items having drop glue.
///
/// This is mainly useful to allow generic functions to return tuples without being then stuck
/// without a way to pull them back apart.
///
/// # Example
/// ```
/// use const_util::*;
/// const fn pair_to_arr<T>(pair: (T, T)) -> [T; 2] {
///     destruct_tuple! { a, b in pair }
///     [a, b]
/// }
/// assert_eq!(
///     pair_to_arr((String::from("ABC"), String::new())),
///     ["ABC", ""],
/// );
/// ```
#[macro_export]
macro_rules! destruct_tuple {
    ($($field:ident),* in $tup:expr) => {
        let __tup: ($($crate::__infer!($field),)*) = $tup;
        let __tup = $crate::__mac::core::mem::ManuallyDrop::new(__tup);
        let ($($field),*) = $crate::mem::man_drop_ref(&__tup);
        // SAFETY: The tuple is forgotten after this
        $(let $field = unsafe { $crate::__mac::core::ptr::read($field) };)*
    };
}

#[doc(hidden)]
pub mod __mac {
    pub use core;
}
