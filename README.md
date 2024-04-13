# generic-statics

**This crate is experimental and might not be fully sound. Use at your own risk.**

A workaround for missing generic statics in Rust.

```rust
use std::{ptr, sync::atomic::{AtomicPtr, Ordering}};

// This code doesn't work.
static A<T>: AtomicPtr<T> = AtomicPtr::new(ptr::null_mut());
let a = A::<usize>.load(Ordering::Relaxed);
```

With `generic-statics`:

```rust
use std::sync::atomic::{AtomicPtr, Ordering};
use generic_statics::{define_namespace, Namespace};

define_namespace!(Test);

// This works.
let a = Test::static_generic::<AtomicPtr<usize>>().load(Ordering::Relaxed);
```

## Caveats

This crate is nightly only and relies on `#![feature(asm_const)]` (As of 2024-04-10, stabilization of that feature is blocked on `feature(inline_const)`).

The generic statics provided by this crate use static allocation (i.e. no dynamic allocation at runtime) and is _almost_ zero-cost (aside from some inline asm instructions for computing the static address).

However, this crate only offers best-effort stable addresses:

```rust
use generic_statics::{define_namespace, Namespace};

define_namespace!(Test);

// This is *not* guaranteed but in most cases this will work just fine.
assert_eq!(
    Test::static_generic::<usize>() as *const _,
    Test::static_generic::<usize>() as *const _
);
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
- Windows `x86_64`
