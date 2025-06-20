use core::{
    mem::ManuallyDrop,
    ptr::{self, NonNull},
};

/// Gets a reference to the contents of a `ManuallyDrop`.
///
/// This function is a const version of the `Deref` implementation of `ManuallyDrop`.
pub const fn man_drop_ref<T>(man: &ManuallyDrop<T>) -> &T {
    // SAFETY: repr(transparent)
    unsafe { &*ptr::from_ref(man).cast() }
}
/// Gets a reference to the contents of a `ManuallyDrop`.
///
/// This function is a const version of the `DerefMut` implementation of `ManuallyDrop`.
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
pub const fn nonnull_from<T: ?Sized>(src: impl hidden::Reference<Referee = T>) -> NonNull<T> {
    const fn doit<T: ?Sized, R: hidden::Reference<Referee = T>>(src: R) -> NonNull<R::Referee> {
        let ptr: *mut T = {
            let src = ManuallyDrop::new(src);
            if R::MUTABLE {
                // SAFETY: &mut T to *mut T transmute
                let ptr: *mut T = unsafe { core::mem::transmute_copy(&src) };
                ptr
            } else {
                // SAFETY: &T to *const T transmute
                let ptr: *const T = unsafe { core::mem::transmute_copy(&src) };
                ptr.cast_mut()
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
