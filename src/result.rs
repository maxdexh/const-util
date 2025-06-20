use crate::mem::man_drop_ref;
use core::{mem::ManuallyDrop, ptr};

/// Const version of [`Result::expect`] without the [`Debug`] formatting.
///
/// Can also be used in combination with [`Result::is_ok`] to match the result.
///
/// # Example
/// ```
/// use const_util::result::expect_ok;
/// const fn double<E>(res: Result<i32, E>) -> Result<i32, E> {
///     if res.is_ok() {
///         Ok(2 * expect_ok(res, "Unreachable"))
///     } else {
///         res
///     }
/// }
/// assert_eq!(double(Ok::<_, String>(1)), Ok(2));
/// ```
#[track_caller]
pub const fn expect_ok<T, E>(res: Result<T, E>, message: &str) -> T {
    let res = ManuallyDrop::new(res);
    match man_drop_ref(&res) {
        // SAFETY: Reading from behind a ManuallyDrop that is not used afterwards
        Ok(ok) => unsafe { ptr::read(ok) },
        Err(_) => {
            let _res = ManuallyDrop::into_inner(res);
            panic!("{}", message)
        }
    }
}
/// Const version of [`Result::unwrap`] without the [`Debug`] formatting.
///
/// Can also be used in combination with [`Result::is_ok`] to match the result.
///
/// # Example
/// ```
/// use const_util::result::unwrap_ok;
/// const fn double<E>(res: Result<i32, E>) -> Result<i32, E> {
///     if res.is_ok() {
///         Ok(2 * unwrap_ok(res))
///     } else {
///         res
///     }
/// }
/// assert_eq!(double(Ok::<_, String>(1)), Ok(2));
/// ```
#[track_caller]
pub const fn unwrap_ok<T, E>(res: Result<T, E>) -> T {
    expect_ok(res, "Attempted to call `unwrap_ok` on an `Err` variant")
}
/// Const version of [`Result::expect_err`] without the [`Debug`] formatting.
///
/// Can also be used in combination with [`Result::is_err`] to match the result.
///
/// # Example
/// ```
/// use const_util::result::expect_err;
/// const fn double<T>(res: Result<T, i32>) -> Result<T, i32> {
///     if res.is_err() {
///         Err(2 * expect_err(res, "Unreachable"))
///     } else {
///         res
///     }
/// }
/// assert_eq!(double(Err::<String, _>(1)), Err(2));
/// ```
#[track_caller]
pub const fn expect_err<T, E>(res: Result<T, E>, message: &str) -> E {
    let res = ManuallyDrop::new(res);
    match man_drop_ref(&res) {
        // SAFETY: Reading from behind a ManuallyDrop that is not used afterwards
        Err(err) => unsafe { ptr::read(err) },
        Ok(_) => {
            let _res = ManuallyDrop::into_inner(res);
            panic!("{}", message)
        }
    }
}
/// Const version of [`Result::unwrap_err`] without the [`Debug`] formatting.
///
/// Can also be used in combination with [`Result::is_err`] to match the result.
///
/// # Example
/// ```
/// use const_util::result::unwrap_err;
/// const fn double<T>(res: Result<T, i32>) -> Result<T, i32> {
///     if res.is_err() {
///         Err(2 * unwrap_err(res))
///     } else {
///         res
///     }
/// }
/// assert_eq!(double(Err::<String, _>(1)), Err(2));
/// ```
#[track_caller]
pub const fn unwrap_err<T, E>(res: Result<T, E>) -> E {
    expect_err(res, "Attempted to call `unwrap_err` on an `Ok` variant")
}
