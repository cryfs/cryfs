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
// Creating
let mut guard = AsyncDropGuard::new(my_value);

// Using (transparent via Deref)
guard.do_something();

// Cleanup (REQUIRED before dropping)
guard.async_drop().await?;
```

## The AsyncDrop Trait

```rust
#[async_trait]
pub trait AsyncDrop {
    type Error: Debug;
    async fn async_drop_impl(&mut self) -> Result<(), Self::Error>;
}
```

## Essential Rules

| Rule | Description |
|------|-------------|
| **Always call async_drop()** | Every `AsyncDropGuard` must have `async_drop()` called |
| **Factory methods return guards** | `fn new() -> AsyncDropGuard<Self>`, never plain `Self` |
| **Types with guard members impl AsyncDrop** | Delegate to member async_drops |
| **Use the macro when possible** | `with_async_drop_2!` handles cleanup automatically |
| **Panics are exceptions** | It's OK to skip async_drop on panic paths |

## The `with_async_drop_2!` Macro

Automatically calls `async_drop()` on scope exit:

```rust
let resource = get_resource().await?;
with_async_drop_2!(resource, {
    // Use resource here
    resource.do_work().await?;
    Ok(result)
})
```

## Additional References

- [patterns.md](patterns.md) - Implementation patterns and examples
- [gotchas.md](gotchas.md) - Common mistakes and how to avoid them
- [helpers.md](helpers.md) - Helper types (AsyncDropArc, AsyncDropHashMap, etc.)

## Location

Implementation: `crates/utils/src/async_drop/`
