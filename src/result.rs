use crate::mem::man_drop_ref;
use core::{mem::ManuallyDrop, ptr};

/// Const version of [`Result::expect`] without the [`Debug`] formatting.
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
#[track_caller]
pub const fn unwrap_ok<T, E>(res: Result<T, E>) -> T {
    expect_ok(res, "Attempted to call `unwrap_ok` on an `Err` variant")
}
/// Const version of [`Result::expect_err`] without the [`Debug`] formatting.
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
#[track_caller]
pub const fn unwrap_err<T, E>(res: Result<T, E>) -> E {
    expect_err(res, "Attempted to call `unwrap_err` on an `Ok` variant")
}
