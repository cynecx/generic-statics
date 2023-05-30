# static-generics

**This crate is experimental and might not be fully sound. Use at your own risk.**

A workaround for missing static generics in Rust.

```rust
use std::{ptr, sync::atomic::{AtomicPtr, Ordering}};

// This code doesn't work.
static A<T>: AtomicPtr<T> = AtomicPtr::new(ptr::null_mut());
let a = A::<usize>.load(Ordering::Relaxed);
```

With `static-generics`:

```rust
use std::sync::atomic::{AtomicPtr, Ordering};
use static_generics::static_generic;

// This works.
let a = static_generic::<AtomicPtr<usize>>().load(Ordering::Relaxed);
```

## Caveats

This crate is nightly only and relies on `#![feature(asm_const)]`.

The static generics provided by this crate use static allocation (no dynamic allocation at runtime) and is almost zero-cost (aside from some inline asm instructions for computing the static address).

However, this crate only offers best-effort stable addresses:

```rust
use static_generics::static_generic;
assert_eq!(static_generic::<usize>() as *const _, static_generic::<usize>() as *const _);
```

The used approach relies on inline assembly to instantiate/reserve static data for each monomorphized variant of the function.
Unfortunately inlining will return a different version of the data and thus will not return stable addresses.
However, `static_generic` is marked `#[inline(never)]` which should provide stable addresses in most situations
(Note that `#[inline(never)]` is just a hint to the compiler and doesn't guarantee anything).

Only "zeroable" types are allowed for now due to inline asm restrictions.

This crate only supports these targets for now:

- macOS `x86_64`, `aarch64`
- Linux `x86_64`, `aarch64`
- FreeBSD `x86_64`, `aarch64`

Windows isn't support due to missing support for some inline asm directives (`.pushsection` and `.popsection`).
