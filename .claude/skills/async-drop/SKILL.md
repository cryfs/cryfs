---
name: async-drop
description: Guide to the AsyncDrop pattern for async cleanup in Rust. Use when working with AsyncDropGuard, implementing AsyncDrop trait, or handling async resource cleanup.
---

# AsyncDrop Pattern Guide

The AsyncDrop pattern enables async cleanup for types that hold resources requiring asynchronous teardown (network connections, file handles, background tasks, etc.).

## Core Concept

Rust's `Drop` trait is synchronous, but sometimes cleanup needs to be async. The AsyncDrop pattern solves this by:

1. Wrapping values in `AsyncDropGuard<T>`
2. Requiring explicit `async_drop().await` calls
3. Panicking if cleanup is forgotten

## Quick Reference

```rust
// Creating (no `mut` needed unless you mutate the value)
let guard = AsyncDropGuard::new(my_value);

// Using (transparent via Deref)
guard.do_something();

// Cleanup (REQUIRED before dropping). Consumes the guard:
// using `guard` after this line is a compile error.
guard.async_drop().await?;
```

## The AsyncDrop Trait

```rust
pub trait AsyncDrop {
    type Error: Debug;
    fn async_drop_impl(self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
```

Implement it as a plain `async fn async_drop_impl(self)` without `#[async_trait]`.
`self` is taken by value, so destructure it to move `AsyncDropGuard` members out
and drop them. List every field (`name: _` for the ones that don't need dropping)
instead of using `..`: that way adding a field later is a compile error here, which
forces whoever adds an `AsyncDropGuard` field to decide how to drop it.

```rust
impl AsyncDrop for MyType {
    type Error = anyhow::Error;

    async fn async_drop_impl(self) -> Result<(), Self::Error> {
        let Self { connection, cache, config: _ } = self;
        cache.async_drop().await?;
        connection.async_drop().await?;
        Ok(())
    }
}
```

## Essential Rules

| Rule | Description |
|------|-------------|
| **Always call async_drop()** | Every `AsyncDropGuard` must have `async_drop()` called |
| **async_drop() consumes the guard** | Use-after-drop and double-drop are compile errors |
| **Factory methods return guards** | `fn new() -> AsyncDropGuard<Self>`, never plain `Self` |
| **Types with guard members impl AsyncDrop** | `async fn async_drop_impl(self)`: destructure `self`, drop members |
| **No `#[async_trait]` on AsyncDrop impls** | The trait uses native `async fn` in traits |
| **Use the macro when possible** | `with_async_drop!` handles cleanup automatically |
| **Panics are exceptions** | It's OK to skip async_drop on panic paths |

## The `with_async_drop!` Macro

Automatically calls `async_drop()` on scope exit:

```rust
let resource = get_resource().await?;
with_async_drop!(resource, {
    // Use resource here
    resource.do_work().await?;
    Ok(result)
})
```

Several independent guards can be listed; they are dropped concurrently afterward:

```rust
with_async_drop!(source, dest, {
    move_entry(&source, &dest).await
})
```

To drop several independent guards without running a block, use `async_drop_all((a, b, c)).await?`.

## Additional References

- [patterns.md](patterns.md) - Implementation patterns and examples
- [gotchas.md](gotchas.md) - Common mistakes and how to avoid them
- [helpers.md](helpers.md) - Helper types (AsyncDropArc, AsyncDropHashMap, etc.)

## Location

Implementation: `crates/utils/src/async_drop/`
