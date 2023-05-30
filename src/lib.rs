#![feature(asm_const)]

//! A workaround for missing static generics in Rust.
//!
//! **This crate is experimental and might not be fully sound. Use at your own risk.**
//!
//! The core functionality is provided by [`static_generic`].
//!
//! ```rust
//! use std::sync::atomic::{AtomicPtr, Ordering};
//! use static_generics::static_generic;
//!
//! let a = static_generic::<AtomicPtr<usize>>().load(Ordering::Relaxed);
//! ```
//!
//! ## Caveats and Limitations
//!
//! This crate is nightly only and relies on `#![feature(asm_const)]`.
//!
//! The static generics provided by this crate use static allocation
//! (no dynamic allocation at runtime) and is almost zero-cost
//! (aside from some inline asm instructions for computing the static address).
//!
//! However, this crate only offers best-effort stable addresses:
//!
//! ```rust
//! static_generic::<usize>() as *const _ == static_generic::<usize>() as *const _
//! ```
//!
//! The used approach relies on inline assembly to instantiate/reserve static data for each monomorphized variant of the function.
//! Unfortunately inlining will return a different version of the data and thus will not return stable addresses.
//! However, `static_generic` is marked `#[inline(never)]` which should provide stable addresses in most situations
//! (Note that `#[inline(never)]` is just a hint to the compiler and doesn't guarantee anything).
//!
//! Only "zeroable" types are allowed for now due to inline asm restrictions.
//!
//! This crate only supports these targets for now:
//!
//! - macOS `x86_64`, `aarch64`
//! - Linux `x86_64`, `aarch64`
//! - FreeBSD `x86_64`, `aarch64`
//!
//! Windows isn't support due to missing support for some inline asm directives (`.pushsection` and `.popsection`).

mod zeroable;

use std::{any::TypeId, mem};

pub use zeroable::Zeroable;

const fn cmp_max(a: usize, b: usize) -> usize {
    if a > b {
        a
    } else {
        b
    }
}

/// The reference returned by [`static_generic`] points to the static global variable for each
/// generic `T` (but are lifetime erased). The static's value is zero-initialized.
///
/// For caveats and limitations, refer to [top-module](crate#caveats-and-limitations).
#[inline(never)]
pub fn static_generic<T: 'static + Zeroable>() -> &'static T {
    let mut addr: *const ();

    // HACK: We have to "use" the generic `T` in some way to force the compiler to emit every
    // instatiation of this function, otherwise rustc might be smart and merge instantiations.
    let type_id = TypeId::of::<T> as *const ();

    #[cfg(all(
        target_arch = "aarch64",
        any(target_os = "macos", target_os = "ios", target_os = "tvos")
    ))]
    unsafe {
        std::arch::asm!(
            "/* {type_id} */",
            "adrp {x}, 1f@PAGE",
            "add {x}, {x}, 1f@PAGEOFF",
            ".pushsection __DATA,__data",
            ".p2align {align}, 0",
            "1: .zero {size}",
            ".popsection",
            size = const { cmp_max(mem::size_of::<T>(), 1) },
            align = const { mem::align_of::<T>().ilog2() },
            type_id = in(reg) type_id,
            x = out(reg) addr,
            options(nostack)
        );
    }

    #[cfg(all(
        target_arch = "aarch64",
        any(target_os = "none", target_os = "linux", target_os = "freebsd")
    ))]
    unsafe {
        std::arch::asm!(
            "/* {type_id} */",
            "adrp {x}, 1f",
            "add {x}, {x}, :lo12:1f",
            ".pushsection .bss.static_generics,\"aw\",@nobits",
            ".p2align {align}, 0",
            "1: .zero {size}",
            ".popsection",
            size = const { cmp_max(mem::size_of::<T>(), 1) },
            align = const { mem::align_of::<T>().ilog2() },
            type_id = in(reg) type_id,
            x = out(reg) addr,
            options(nostack)
        );
    }

    #[cfg(all(
        target_arch = "x86_64",
        any(target_os = "macos", target_os = "ios", target_os = "tvos")
    ))]
    unsafe {
        std::arch::asm!(
            "/* {type_id} */",
            "lea {x}, [rip + 1f]",
            ".pushsection __DATA,__data",
            ".p2align {align}, 0",
            "1: .zero {size}",
            ".popsection",
            size = const { cmp_max(mem::size_of::<T>(), 1) },
            align = const { mem::align_of::<T>().ilog2() },
            type_id = in(reg) type_id,
            x = out(reg) addr,
            options(nostack)
        );
    }

    #[cfg(all(
        target_arch = "x86_64",
        any(target_os = "none", target_os = "linux", target_os = "freebsd")
    ))]
    unsafe {
        std::arch::asm!(
            "/* {type_id} */",
            "lea {x}, [rip + 1f]",
            ".pushsection .bss.static_generics,\"aw\",@nobits",
            ".p2align {align}, 0",
            "1: .zero {size}",
            ".popsection",
            size = const { cmp_max(mem::size_of::<T>(), 1) },
            align = const { mem::align_of::<T>().ilog2() },
            type_id = in(reg) type_id,
            x = out(reg) addr,
            options(nostack)
        );
    }

    unsafe { &*(addr as *const T) }
}

#[cfg(test)]
mod tests {
    use std::{
        assert_ne,
        sync::atomic::{AtomicIsize, AtomicPtr, AtomicUsize, Ordering},
    };

    use super::static_generic;

    #[test]
    fn stable_addr() {
        let a = static_generic::<*const ()>() as *const _;
        let b = static_generic::<*const ()>() as *const _;
        assert_eq!(a, b);

        let d = static_generic::<(AtomicUsize, AtomicUsize, AtomicUsize)>() as *const _;
        let e = static_generic::<(AtomicUsize, AtomicUsize, AtomicUsize)>() as *const _;
        assert_eq!(d, e);

        assert_ne!(a as *const (), d as *const _ as *const ());
    }

    #[test]
    fn unique_address() {
        let a = static_generic::<AtomicUsize>() as *const _ as *const ();
        let b = static_generic::<AtomicIsize>() as *const _ as *const ();
        let c = static_generic::<usize>() as *const _ as *const ();
        let d = static_generic::<AtomicPtr<()>>() as *const _ as *const ();

        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
        assert_ne!(b, c);
        assert_ne!(b, d);
        assert_ne!(c, d);
    }

    #[test]
    fn mutation() {
        let a = static_generic::<AtomicUsize>();
        assert_eq!(a.load(Ordering::Relaxed), 0);
        a.store(42, Ordering::Relaxed);

        let b = static_generic::<AtomicUsize>();
        assert_eq!(b.load(Ordering::Relaxed), 42);

        let a2 = static_generic::<AtomicIsize>();
        assert_eq!(a2.load(Ordering::Relaxed), 0);
    }
}
