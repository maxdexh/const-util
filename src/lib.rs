#![no_std]

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
