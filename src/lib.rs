#![feature(asm_const)]

//! A "workaround" for missing generic statics in Rust.
//!
//! **This crate is experimental and might not be fully sound. Use at your own risk.**
//!
//! The core functionality is provided by [`Namespace::generic_static`].
//!
//! ```rust
//! use std::sync::atomic::{AtomicPtr, Ordering};
//! use generic_statics::{define_namespace, Namespace};
//!
//! define_namespace!(Test);
//!
//! let a = Test::generic_static::<AtomicPtr<usize>>().load(Ordering::Relaxed);
//! ```
//!
//! ## Caveats and Limitations
//!
//! This crate is nightly only and relies on `#![feature(asm_const)]`.
//!
//! The generic statics provided by this crate use static allocation
//! (i.e. no dynamic allocation will occur at runtime) and is almost zero-cost
//! (aside from some inline asm instructions for computing the static address).
//!
//! However, this crate is **only best-effort** (i.e. best-effort stable addresses):
//!
//! ```rust
//! use generic_statics::{define_namespace, Namespace};
//!
//! define_namespace!(Test);
//!
//! // This is *not* guaranteed, but in usual cases this will work just fine.
//! assert_eq!(
//!     Test::generic_static::<usize>() as *const _,
//!     Test::generic_static::<usize>() as *const _
//! );
//! ```
//!
//! The used approach relies on inline assembly to instantiate/reserve static data for each
//! monomorphized variant of the function.
//! Unfortunately inlining will return a different version of the data and thus will not return
//! stable addresses.
//! However, [`Namespace::generic_static`] is marked `#[inline(never)]` which should provide stable
//! addresses in most situations
//! (Note that `#[inline(never)]` is just a hint to the compiler and doesn't guarantee anything).
//!
//! Only "zeroable" types are allowed for now due to inline asm restrictions.
//!
//! This crate only supports these targets for now:
//!
//! - macOS `x86_64`, `aarch64`
//! - Linux `x86_64`, `aarch64`
//! - FreeBSD `x86_64`, `aarch64`
//! - Windows `x86_64`
//!

mod zeroable;

use std::{any::TypeId, mem, ptr};

pub use zeroable::Zeroable;

const fn cmp_max(a: usize, b: usize) -> usize {
    if a > b {
        a
    } else {
        b
    }
}

/// A namespace for generic statics.
///
/// # Safety
///
/// Implementing this trait is not unsafe per-se but you should use the [`define_namespace`]
/// instead.
pub unsafe trait Namespace: 'static + Send + Sync + Copy + Clone {
    /// The returned reference points to the static namespaced global variable for each
    /// generic `T` (but are lifetime erased). The static's value is zero-initialized.
    ///
    /// For caveats and limitations, refer to [top-module](crate#caveats-and-limitations).
    #[inline(never)]
    #[must_use]
    fn generic_static<T: 'static + Zeroable>() -> &'static T {
        #[allow(unused_assignments)]
        let mut addr: *const () = ptr::null();

        // HACK: We have to "use" the generic `T` in some way to force the compiler to emit every
        // instatiation of this function, otherwise rustc might be smart and merge instantiations.
        let type_id = TypeId::of::<(Self, T)> as *const ();

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
                ".pushsection .bss.generic_statics,\"aw\",@nobits",
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
                ".pushsection .bss.generic_statics,\"aw\",@nobits",
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

        #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
        unsafe {
            std::arch::asm!(
                "/* {type_id} */",
                "lea {x}, [rip + 1f]",
                ".pushsection .bss.generic_statics,\"bw\"",
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

        #[cfg(not(any(
            target_os = "none",
            target_os = "linux",
            target_os = "freebsd",
            target_os = "macos",
            target_os = "ios",
            target_os = "tvos",
            target_os = "windows",
        )))]
        std::compile_error!("static-generics is not supported on this platform");

        assert!(!addr.is_null(), "unsupported platform");

        unsafe { &*addr.cast::<T>() }
    }
}

#[macro_export]
macro_rules! define_namespace {
    ($vis:vis $name:ident) => {
        #[derive(Debug, Copy, Clone)]
        $vis struct $name;

        unsafe impl $crate::Namespace for $name {}
    };
}

#[cfg(test)]
mod tests {
    use std::{
        assert_ne,
        marker::PhantomData,
        sync::atomic::{AtomicIsize, AtomicPtr, AtomicUsize, Ordering},
    };

    use super::Namespace;

    define_namespace!(pub Test);

    #[test]
    fn stable_addr() {
        let a = Test::generic_static::<*const ()>() as *const _;
        let b = Test::generic_static::<*const ()>() as *const _;
        assert_eq!(a, b);

        let d = Test::generic_static::<(AtomicUsize, AtomicUsize, AtomicUsize)>() as *const _;
        let e = Test::generic_static::<(AtomicUsize, AtomicUsize, AtomicUsize)>() as *const _;
        assert_eq!(d, e);

        assert_ne!(a as *const (), d as *const _ as *const ());
    }

    #[test]
    fn unique_address() {
        let a = Test::generic_static::<AtomicUsize>() as *const _ as *const ();
        let b = Test::generic_static::<AtomicIsize>() as *const _ as *const ();
        let c = Test::generic_static::<usize>() as *const _ as *const ();
        let d = Test::generic_static::<AtomicPtr<()>>() as *const _ as *const ();

        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
        assert_ne!(b, c);
        assert_ne!(b, d);
        assert_ne!(c, d);
    }

    #[test]
    fn unique_address_dyn() {
        trait Foo<A: 'static> {}

        let a = Test::generic_static::<PhantomData<dyn Foo<usize>>>() as *const _ as *const ();
        let b = Test::generic_static::<PhantomData<dyn Foo<isize>>>() as *const _ as *const ();
        let c = Test::generic_static::<PhantomData<dyn Foo<()>>>() as *const _ as *const ();

        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(b, c);
    }

    #[test]
    fn mutation() {
        let a = Test::generic_static::<AtomicUsize>();
        assert_eq!(a.load(Ordering::Relaxed), 0);
        a.store(42, Ordering::Relaxed);

        let b = Test::generic_static::<AtomicUsize>();
        assert_eq!(b.load(Ordering::Relaxed), 42);

        let a2 = Test::generic_static::<AtomicIsize>();
        assert_eq!(a2.load(Ordering::Relaxed), 0);
    }
}
