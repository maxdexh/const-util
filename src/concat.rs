//! Functions for concatenating slices

use crate::Const;

struct TwoValues<A, B>(A, B);
impl<'a, T: ?Sized + 'a, A: Const<Type = &'a T>, B: Const<Type = &'a T>> Const for TwoValues<A, B> {
    type Type = &'a [&'a T];
    const VALUE: Self::Type = &[A::VALUE, B::VALUE];
}
/// Concats two `str`s into a single `&'static str` at compile time
pub const fn concat_strs2<'a, Lhs: Const<Type = &'a str>, Rhs: Const<Type = &'a str>>(
) -> &'static str {
    concat_strs::<TwoValues<Lhs, Rhs>>()
}

/// Concats two `[u8]`s into a single `&'static [u8]` at compile time
pub const fn concat_bytes2<'a, Lhs: Const<Type = &'a [u8]>, Rhs: Const<Type = &'a [u8]>>(
) -> &'static [u8] {
    concat_bytes::<TwoValues<Lhs, Rhs>>()
}

/// Concats a collection of `&str`s into a single `&'static str` at compile time.
///
/// # Example
/// ```
/// use const_util::{Const, concat::concat_strs};
/// struct MyStrings;
/// impl Const for MyStrings {
///     type Type = &'static [&'static str];
///     const VALUE: Self::Type = &[
///         "Odd",
///         "Even",
///         "Schmeven",
///     ];
/// }
/// assert_eq!(
///     concat_strs::<MyStrings>(),
///     "OddEvenSchmeven",
/// )
/// ```
pub const fn concat_strs<'a, Strs: Const<Type = &'a [&'a str]>>() -> &'static str {
    struct ToBytes<C>(C);
    impl<'a, C: Const<Type = &'a [&'a str]>> Const for ToBytes<C> {
        type Type = &'a [&'a [u8]];
        // SAFETY: https://doc.rust-lang.org/reference/type-layout.html#r-layout.str
        const VALUE: Self::Type = unsafe {
            let strs = crate::value_of::<C>();
            core::slice::from_raw_parts(strs.as_ptr().cast(), strs.len())
        };
    }
    match core::str::from_utf8(concat_bytes::<ToBytes<Strs>>()) {
        Ok(s) => s,
        Err(_) => unreachable!(),
    }
}

#[rustversion::since(1.87)]
const fn copy_from_slice<T: Copy>(src: &[T], dst: &mut [T]) {
    dst.copy_from_slice(src);
}
#[rustversion::before(1.87)]
const fn copy_from_slice<T: Copy>(src: &[T], dst: &mut [T]) {
    assert!(src.len() == dst.len());
    // SAFETY: T: Copy. This is literally how copy_from_slice is implemented.
    unsafe {
        core::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }
}

macro_rules! generate_slice_concat {
    ($T:ty, $input:ty, $default:expr) => {{
        use generic_upper_bound as gub;
        struct Concat<C>(C);
        gub::impl_accept_upper_bound! {
            impl{'a, C: Const<Type = &'a [&'a [$T]]>} Concat<C>;
            const DESIRED_GENERIC: usize = {
                let mut slices = crate::value_of::<C>();
                let mut out = 0;
                while let Some((first, rest)) = slices.split_first() {
                    out += first.len();
                    slices = rest;
                }
                out
            };
            const EVAL<const N: usize>: &'static [$T] = &{
                let mut out = [const { $default }; N];
                let mut out_slice: &mut [_] = &mut out;
                let mut slices = crate::value_of::<C>();
                while let Some((first, rest)) = slices.split_first() {
                    let lhs;
                    (lhs, out_slice) = out_slice.split_at_mut(first.len());
                    copy_from_slice(first, lhs);
                    slices = rest;
                }
                out
            };
        }
        gub::eval_with_upper_bound::<Concat<$input>>()
            .split_at(gub::desired_generic::<Concat<$input>>())
            .0
    }};
}

/// Concats a collection of `&[u8]`s into a single `&'static [u8]` at compile time.
///
/// # Example
/// Analogous to [`concat_strs`].
pub const fn concat_bytes<'a, Bytes: Const<Type = &'a [&'a [u8]]>>() -> &'static [u8] {
    generate_slice_concat!(u8, Bytes, 0)
}
