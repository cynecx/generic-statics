use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    sync::atomic::{
        AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicPtr, AtomicU16,
        AtomicU32, AtomicU64, AtomicU8, AtomicUsize,
    },
};

/// Types that can be safely "zero-initialized".
///
/// ## Safety
///
/// The type this trait is implemented on must:
///
/// - be inhabited (no `!`)
/// - have a state where all bits are all zeroes
///
/// ## Notes
///
/// Integral types (`i32`, `i64`, ...), and some other type that fulfill the above safety
/// requirements have built-in impls that are provided by this crate.
///
pub unsafe trait Zeroable: Sized {}

macro_rules! impl_integers {
    ($t:ty) => {
        unsafe impl Zeroable for $t {}
    };
    ($t1:ty, $($tr:ty),+) => {
        impl_integers!($t1);
        impl_integers!($($tr),+);
    }
}

impl_integers!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, isize, usize, bool);

impl_integers!(
    AtomicBool,
    AtomicI16,
    AtomicI32,
    AtomicI64,
    AtomicI8,
    AtomicIsize,
    AtomicU16,
    AtomicU32,
    AtomicU64,
    AtomicU8,
    AtomicUsize
);

unsafe impl<T> Zeroable for AtomicPtr<T> {}

unsafe impl<T: Sized> Zeroable for *const T {}
unsafe impl<T: Sized> Zeroable for *mut T {}

unsafe impl<T> Zeroable for MaybeUninit<T> {}

unsafe impl<T: Zeroable> Zeroable for ManuallyDrop<T> {}
unsafe impl<T: Zeroable> Zeroable for UnsafeCell<T> {}
unsafe impl<T: ?Sized> Zeroable for PhantomData<T> {}

unsafe impl<T: Zeroable, const N: usize> Zeroable for [T; N] {}

macro_rules! impl_tuples {
    ($t1:ident) => {
        unsafe impl<$t1: Zeroable> Zeroable for ($t1,) {}
    };
    ($t1:ident, $($tr:ident),+) => {
        impl_tuples!(@impl $t1, $($tr),+);
        impl_tuples!($($tr),+);
    };
    (@impl $($t:ident),+) => {
        unsafe impl<$($t),+> Zeroable for ($($t),+)
        where
            $($t: Zeroable),+
        {}
    };
}

impl_tuples!(A, B, C, D, E, F, G, H);
