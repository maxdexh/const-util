[![Crates.io](https://img.shields.io/crates/v/generic-upper-bound.svg)](https://crates.io/crates/generic-upper-bound)
[![Documentation](https://docs.rs/generic-upper-bound/badge.svg)](https://docs.rs/generic-upper-bound)
[![Rust](https://img.shields.io/badge/rust-1.83.0%2B-blue.svg?maxAge=3600)](https://github.com/rust-lang/generic-upper-bound)

<!-- cargo-rdme start -->

Provides stable const implementations for some things missing from the standard library.

Currently implemented are
- Functions in [`result`](https://docs.rs/const-util/latest/const_util/result/) to unwrap [`Result`](core::result::Result)s with generics or drop glue
- Functions in the [`concat`](https://docs.rs/const-util/latest/const_util/concat/) module to concat const strings and byte slices.
- [`destruct_tuple`](https://docs.rs/const-util/latest/const_util/macro.destruct_tuple.html) to destructure tuples with generic types or types with drop glue in them
- [`nonnull_from`](https://docs.rs/const-util/latest/const_util/mem/fn.nonnull_from.html) to create [`NonNull`](core::ptr::NonNull)s from mutable and regular references
  conveniently
- [`man_drop_ref`](https://docs.rs/const-util/latest/const_util/mem/fn.man_drop_ref.html)/[`man_drop_mut`](https://docs.rs/const-util/latest/const_util/mem/fn.man_drop_mut.html) as a workaround for the lack of const
  [`Deref`](core::ops::Deref) implementations

<!-- cargo-rdme end -->
