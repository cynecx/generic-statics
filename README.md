# static-generics

A workaround for missing static generics in Rust.

```rust
// This code doesn't work.

static A<T>: AtomicPtr<T> = AtomicPtr::new(ptr::null_mut());

let a = A::<usize>.load(Ordering::Relaxed);
```

With `static-generic`:

```rust
// This works.

let a = static_generic::<AtomicPtr<usize>>().load(Ordering::Relaxed);
```

## Caveats

The static generics provided by this crate use static allocation (no dynamic allocation at runtime) and is almost zero-cost (aside from some inline asm instructions for computing the static address).

However, this crate only offers best-effort stable addresses:

```rust
static_generic::<usize>() as *const _ == static_generic::<usize>() as *const _
```

The used approach relies on inline assembly to instantiate/reserve static data for each monomorphized variant of the function.
Unfortunately inlining will return a different version of the data and thus will not return stable addresses.
However, `static_generic` is marked `#[inline(never)]` which should provide stable addresses in most situations
(Note that `#[inline(never)]` is just a hint to the compiler and doesn't guarantee anything).

Only zeroable types are allowed for now due to inline asm restrictions.

This crate only supports these targets for now:

- macOS `x86_64`, `aarch64`
- Linux `x86_64`, `aarch64`
- FreeBSD `x86_64`, `aarch64`
- Windows `x86_64`
