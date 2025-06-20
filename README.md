# const-util

Provides stable const implementations for some things missing from the standard library.

Currently implemented are
- Functions in [`result`] to unwrap results with generics or drop glue
- Functions in the [`concat`] module to concat const strings and byte slices.
- [`destruct_tuple`] to destructure tuples with generic types or types with drop glue in them
- [`mem::nonnull_from`] to create [`core::ptr::NonNull`]s from mutable and regular references
  conveniently
- [`mem::man_drop_ref`]/[`mem::man_drop_mut`] as a workaround for the lack of const
  [`Deref`](core::ops::Deref) implementations

License: MIT
