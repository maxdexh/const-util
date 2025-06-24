use core::ops::{Bound, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
mod hidden {
    use super::*;

    #[repr(u8)]
    pub enum IndexKind {
        Range,
        RangeInc,
        RangeFull,
        RangeFrom,
        RangeTo,
        RangeToInc,
        BoundPair,
    }
    /// # Safety
    /// `KIND` must be unique such that we can transmute back to `Self` based on it.
    pub unsafe trait RangeIndex {
        const KIND: IndexKind;
    }
    // SAFETY: `KIND` is unique
    unsafe impl RangeIndex for Range<usize> {
        const KIND: IndexKind = IndexKind::Range;
    }
    // SAFETY: `KIND` is unique
    unsafe impl RangeIndex for RangeInclusive<usize> {
        const KIND: IndexKind = IndexKind::RangeInc;
    }
    // SAFETY: `KIND` is unique
    unsafe impl RangeIndex for RangeToInclusive<usize> {
        const KIND: IndexKind = IndexKind::RangeToInc;
    }
    // SAFETY: `KIND` is unique
    unsafe impl RangeIndex for RangeFrom<usize> {
        const KIND: IndexKind = IndexKind::RangeFrom;
    }
    // SAFETY: `KIND` is unique
    unsafe impl RangeIndex for RangeFull {
        const KIND: IndexKind = IndexKind::RangeFull;
    }
    // SAFETY: `KIND` is unique
    unsafe impl RangeIndex for RangeTo<usize> {
        const KIND: IndexKind = IndexKind::RangeTo;
    }
    // SAFETY: `KIND` is unique
    unsafe impl RangeIndex for (Bound<usize>, Bound<usize>) {
        const KIND: IndexKind = IndexKind::BoundPair;
    }
}
use hidden::*;

use crate::mem::nonnull_from;
use core::ptr::NonNull;
const fn transmute_generic<Src: RangeIndex, Dst: RangeIndex>(src: Src) -> Dst {
    assert!(const { Src::KIND as u8 == Dst::KIND as u8 });
    let src = core::mem::ManuallyDrop::new(src);
    // SAFETY: `KIND` uniquely identifies the implementor, meaning that `Src` and `Dst` are the
    // same type, so this is a safe transmute from `ManuallyDrop<T>` to `T`
    unsafe { core::mem::transmute_copy(&src) }
}

const fn into_range<R: RangeIndex>(range: R, len: usize) -> Option<Range<usize>> {
    Some(match R::KIND {
        IndexKind::Range => transmute_generic(range),
        IndexKind::RangeInc => {
            let range: RangeInclusive<usize> = transmute_generic(range);
            let Some(end) = range.end().checked_add(1) else {
                return None;
            };
            *range.start()..end
        }
        IndexKind::RangeFull => {
            let _: RangeFull = transmute_generic(range);
            0..len
        }
        IndexKind::RangeFrom => {
            let r: RangeFrom<usize> = transmute_generic(range);
            r.start..len
        }
        IndexKind::RangeTo => {
            let r: RangeTo<usize> = transmute_generic(range);
            0..r.end
        }
        IndexKind::RangeToInc => {
            let range: RangeToInclusive<usize> = transmute_generic(range);
            let Some(end) = range.end.checked_add(1) else {
                return None;
            };
            0..end
        }
        IndexKind::BoundPair => {
            let (start, end) = transmute_generic(range);
            (match start {
                Bound::Included(start) => start,
                Bound::Excluded(start) => match start.checked_add(1) {
                    Some(it) => it,
                    None => return None,
                },
                Bound::Unbounded => 0,
            })..match end {
                Bound::Included(end) => match end.checked_add(1) {
                    Some(it) => it,
                    None => return None,
                },
                Bound::Excluded(end) => end,
                Bound::Unbounded => len,
            }
        }
    })
}

/// # Safety
/// `slice` must come from a mutable or immutable reference. The returned pointer is valid for
/// reborrowing as a subslice with the same mutability as the original reference.
const unsafe fn slice_get_nonnull<T, R>(slice: NonNull<[T]>, range: R) -> Option<NonNull<[T]>>
where
    R: RangeIndex,
{
    let Some(Range { start, end }) = into_range(range, slice.len()) else {
        return None;
    };
    let new_len = match usize::checked_sub(end, start) {
        Some(it) if end <= slice.len() => it,
        _ => return None,
    };
    // SAFETY: `slice` came from a reference and `start <= end < slice.len()`, so the pointer
    // addition is in-bounds. The returned pointer can reborrowed as a valid subslice with the
    // same mutability because `start + new_len = start + end - start = end < slice.len()`.
    Some(unsafe { NonNull::slice_from_raw_parts(slice.cast::<T>().add(start), new_len) })
}
/// # Safety
/// `slice` must come from a mutable or immutable reference. The returned pointer is valid for
/// reborrowing as a subslice with the same mutability as the original reference.
#[track_caller]
const unsafe fn slice_index_nonnull<T, R>(slice: NonNull<[T]>, range: R) -> NonNull<[T]>
where
    R: RangeIndex,
{
    #[track_caller]
    #[cold]
    const fn build_msg_panic<const MSG_LEN: usize>(
        msg_lhs: &str,
        left_usize: usize,
        msg_mid: &str,
        right_usize: usize,
    ) -> ! {
        const fn write_str_usize(mut n: usize, mut to: &mut [u8]) -> &mut [u8] {
            while let [slot, rem @ ..] = to {
                let brk = n < 10;

                *slot = (n % 10) as u8 + b'0';
                n /= 10;
                to = rem;

                if brk {
                    break;
                }
            }
            to
        }
        let mut msg = [0; MSG_LEN];
        let (lhs, rem) = msg.split_at_mut(msg_lhs.len());
        lhs.copy_from_slice(msg_lhs.as_bytes());
        let rem = write_str_usize(left_usize, rem);
        let (mid, rem) = rem.split_at_mut(msg_mid.len());
        mid.copy_from_slice(msg_mid.as_bytes());
        let rem = write_str_usize(right_usize, rem);
        let rem_len = rem.len();
        match core::str::from_utf8(msg.split_at(MSG_LEN - rem_len).0) {
            Ok(msg) => panic!("{}", msg),
            Err(_) => unreachable!(),
        }
    }
    const USIZE_STR_LEN: usize = {
        let mut len = 1;
        let mut n = usize::MAX;
        while n >= 10 {
            n /= 10;
            len += 1;
        }
        len
    };

    let Some(Range { start, end }) = into_range(range, slice.len()) else {
        const fn overflow_fail() -> ! {
            panic!("attempted to index slice after maximum allowed usize")
        }
        overflow_fail()
    };
    let Some(new_len) = usize::checked_sub(end, start) else {
        #[track_caller]
        #[cold]
        const fn bounds_order_fail(start: usize, end: usize) -> ! {
            const LHS: &str = "slice index starts at ";
            const MID: &str = " but ends at ";
            const MSG_LEN: usize = LHS.len() + MID.len() + 2 * USIZE_STR_LEN;
            build_msg_panic::<MSG_LEN>(LHS, start, MID, end)
        }
        bounds_order_fail(start, end)
    };
    if end > slice.len() {
        #[track_caller]
        #[cold]
        const fn end_too_large_fail(index: usize, len: usize) -> ! {
            const LHS: &str = "range end index ";
            const MID: &str = " is out of range for slice of length ";
            const MSG_LEN: usize = LHS.len() + MID.len() + 2 * USIZE_STR_LEN;
            build_msg_panic::<MSG_LEN>(LHS, index, MID, len)
        }
        end_too_large_fail(end, slice.len());
    }
    // SAFETY: `slice` came from a reference and `start <= end < slice.len()`, so the pointer
    // addition is in-bounds. The returned pointer can reborrowed as a valid subslice with the
    // same mutability because `start + new_len = start + end - start = end < slice.len()`.
    unsafe { NonNull::slice_from_raw_parts(slice.cast::<T>().add(start), new_len) }
}

/// Const equivalent of [`<[T]>::get`](slice::get) for ranges.
pub const fn slice_get<T, I>(slice: &[T], index: I) -> Option<&[T]>
where
    I: RangeIndex,
{
    // SAFETY: `slice` comes from a reference and the resulting pointer is a valid
    // subslice with the same mutability
    unsafe {
        match slice_get_nonnull(nonnull_from(slice), index) {
            Some(r) => Some(r.as_ref()),
            None => None,
        }
    }
}
/// Const equivalent of [`<[T]>::index`](slice::index) for ranges.
pub const fn slice_index<T, I>(slice: &[T], index: I) -> &[T]
where
    I: RangeIndex,
{
    // SAFETY: `slice` comes from a reference and the resulting pointer is a valid
    // subslice with the same mutability
    unsafe { slice_index_nonnull(nonnull_from(slice), index).as_ref() }
}

/// Const equivalent of [`<[T]>::get_mut`](slice::get_mut) for ranges.
pub const fn slice_get_mut<T, I>(slice: &mut [T], index: I) -> Option<&mut [T]>
where
    I: RangeIndex,
{
    // SAFETY: `slice` comes from a reference and the resulting pointer is a valid
    // subslice with the same mutability
    unsafe {
        match slice_get_nonnull(nonnull_from(slice), index) {
            Some(mut r) => Some(r.as_mut()),
            None => None,
        }
    }
}

/// Const equivalent of [`<[T]>::index_mut`](slice::index_mut) for ranges.
pub const fn slice_index_mut<T, I>(slice: &mut [T], index: I) -> &mut [T]
where
    I: RangeIndex,
{
    // SAFETY: `slice` comes from a reference and the resulting pointer is a valid
    // subslice with the same mutability
    unsafe { slice_index_nonnull(nonnull_from(slice), index).as_mut() }
}

#[test]
fn test() {
    let mut example: Vec<i32> = (0..20).collect();
    let mut example2: Vec<i32> = example.clone();
    let example = &mut *example;
    let example2 = &mut *example2;
    for i in 0..20 {
        for j in 0..20 {
            if i <= j {
                assert_eq!(slice_index_mut(example, i..j), &mut example2[i..j]);
            } else {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    slice_index_mut(example, i..j);
                }))
                .unwrap_err();
            }
            assert_eq!(slice_get_mut(example, i..j), example2.get_mut(i..j));
        }
    }
}
