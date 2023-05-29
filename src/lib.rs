#![feature(asm_const)]

mod zeroable;

use std::mem;

pub use zeroable::Zeroable;

const fn cmp_max(a: usize, b: usize) -> usize {
    if a > b {
        a
    } else {
        b
    }
}

#[inline(never)]
pub fn static_generic<T: 'static + Zeroable>() -> &'static T {
    let mut addr: *const ();

    #[cfg(all(
        target_arch = "aarch64",
        any(target_os = "macos", target_os = "ios", target_os = "tvos")
    ))]
    unsafe {
        std::arch::asm!(
            "adrp {x}, 1f@PAGE",
            "add {x}, {x}, 1f@PAGEOFF",
            ".pushsection __DATA,__data",
            ".p2align {align}, 0",
            "1: .zero {size}",
            ".popsection",
            size = const { cmp_max(mem::size_of::<T>(), 1) },
            align = const { mem::align_of::<T>().ilog2() },
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
            "adrp {x}, 1f",
            "add {x}, {x}, :lo12:1f",
            ".pushsection .bss.static_generics,\"aw\",@nobits",
            ".p2align {align}, 0",
            "1: .zero {size}",
            ".popsection",
            size = const { cmp_max(mem::size_of::<T>(), 1) },
            align = const { mem::align_of::<T>().ilog2() },
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
            "lea {x}, [rip + 1f]",
            ".pushsection __DATA,__data",
            ".p2align {align}, 0",
            "1: .zero {size}",
            ".popsection",
            size = const { cmp_max(mem::size_of::<T>(), 1) },
            align = const { mem::align_of::<T>().ilog2() },
            x = out(reg) addr,
            options(nostack)
        );
    }

    #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
    unsafe {
        std::arch::asm!(
            "lea {x}, [rip + 1f]",
            ".pushsection .section .static_generics,\"dw\"",
            ".p2align {align}, 0",
            "1: .zero {size}",
            ".popsection",
            size = const { cmp_max(mem::size_of::<T>(), 1) },
            align = const { mem::align_of::<T>().ilog2() },
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
            "lea {x}, [rip + 1f]",
            ".pushsection .bss.static_generics,\"aw\",@nobits",
            ".p2align {align}, 0",
            "1: .zero {size}",
            ".popsection",
            size = const { cmp_max(mem::size_of::<T>(), 1) },
            align = const { mem::align_of::<T>().ilog2() },
            x = out(reg) addr,
            options(nostack)
        );
    }

    unsafe { &*(addr as *const T) }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};

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
    fn mutation() {
        let a = static_generic::<AtomicUsize>();
        assert_eq!(a.load(Ordering::Relaxed), 0);
        a.store(42, Ordering::Relaxed);

        let b = static_generic::<AtomicUsize>();
        assert_eq!(b.load(Ordering::Relaxed), 42);

        let a = static_generic::<AtomicIsize>();
        assert_eq!(a.load(Ordering::Relaxed), 0);
    }
}
