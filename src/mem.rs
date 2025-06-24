//! Functions related to [`core::mem`] and [`core::ptr`]

use core::{
    mem::ManuallyDrop,
    ptr::{self, NonNull},
};

/// Gets a reference to the contents of a `ManuallyDrop`.
///
/// This function is a const version of the `Deref` implementation of `ManuallyDrop`.
///
/// # Example
/// ```
/// use const_util::mem::man_drop_ref;
/// use core::mem::ManuallyDrop;
/// assert_eq!(man_drop_ref(&ManuallyDrop::new(1)), &1);
/// ```
pub const fn man_drop_ref<T>(man: &ManuallyDrop<T>) -> &T {
    // SAFETY: repr(transparent)
    unsafe { &*ptr::from_ref(man).cast() }
}
/// Gets a reference to the contents of a `ManuallyDrop`.
///
/// This function is a const version of the `DerefMut` implementation of `ManuallyDrop`.
///
/// # Example
/// ```
/// use const_util::mem::man_drop_mut;
/// use core::mem::ManuallyDrop;
/// assert_eq!(man_drop_mut(&mut ManuallyDrop::new(1)), &mut 1);
/// ```
pub const fn man_drop_mut<T>(man: &mut ManuallyDrop<T>) -> &mut T {
    // SAFETY: repr(transparent)
    unsafe { &mut *ptr::from_mut(man).cast() }
}

/// Converts `&T` or `&mut T` into `NonNull<T>`.
///
/// This function is a const version of the `From<&T>` and `From<&mut T>` implementations of
/// `NonNull<T>`. The returned pointer will have the same mutability as the argument.
///
/// This function is useful to avoid accidentally converting a mutable reference to an immutable
/// one when copying around code.
///
/// # Example
/// ```
/// use const_util::mem::nonnull_from;
/// use core::ptr::NonNull;
///
/// let mut x = 1;
/// assert_eq!(
///     nonnull_from(&x),
///     NonNull::from(&x),
/// );
/// assert_eq!(
///     nonnull_from(&mut x),
///     NonNull::from(&mut x),
/// );
/// unsafe { nonnull_from(&mut x).write(2) };
/// assert_eq!(unsafe { nonnull_from(&x).read() }, 2);
/// ```
pub const fn nonnull_from<T: ?Sized>(src: impl hidden::Reference<Referee = T>) -> NonNull<T> {
    const fn doit<T: ?Sized, R: hidden::Reference<Referee = T>>(src: R) -> NonNull<R::Referee> {
        let ptr: *mut T = {
            let src = ManuallyDrop::new(src);
            if R::MUTABLE {
                // SAFETY: &'a mut T to &'b mut T transmute
                let ptr: &mut T = unsafe { core::mem::transmute_copy(&src) };
                ptr
            } else {
                // SAFETY: &'a T to &'b T transmute
                let ptr: &T = unsafe { core::mem::transmute_copy(&src) };
                ptr::from_ref(ptr).cast_mut()
            }
        };
        // SAFETY: References are non-null
        unsafe { NonNull::new_unchecked(ptr) }
    }
    doit(src)
}
mod hidden {
    /// # Safety
    /// `Self` must be `&Referee` and `MUTABLE = false` or `&mut Referee` and `MUTABLE = true`
    pub unsafe trait Reference: Into<core::ptr::NonNull<Self::Referee>> {
        type Referee: ?Sized;
        const MUTABLE: bool;
    }
    unsafe impl<T: ?Sized> Reference for &T {
        type Referee = T;
        const MUTABLE: bool = false;
    }
    unsafe impl<T: ?Sized> Reference for &mut T {
        type Referee = T;
        const MUTABLE: bool = true;
    }
}
