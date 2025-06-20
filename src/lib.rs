#![no_std]

//! Provides stable const implementations for some things missing from the standard library.
//!
//! Currently implemented are
//! - Functions in the [`concat`] module to concat const strings and byte slices.
//! - [`destruct_tuple`] to destructure tuples with generic types or types with drop glue in them
//! - [`mem::nonnull_from`] to create [`core::ptr::NonNull`]s from mutable and regular references
//!   conveniently
//! - [`mem::man_drop_ref`]/[`mem::man_drop_mut`] as a workaround for the lack of const
//!   [`Deref`](core::ops::Deref) implementations

/// A constant value
pub trait Const {
    /// The type of the constant
    type Type;
    /// The value of the constant
    const VALUE: Self::Type;
}
/// Alias for [`Const::VALUE`].
///
/// Note that functions are evaluated more lazily than associated consts in const contexts by the
/// current compiler.
pub const fn value_of<C: Const>() -> C::Type {
    C::VALUE
}

pub mod concat;
pub mod mem;
pub mod result;

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
