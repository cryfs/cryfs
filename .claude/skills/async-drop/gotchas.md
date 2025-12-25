# AsyncDrop Gotchas

Common mistakes and pitfalls when using the AsyncDrop pattern.

## Gotcha 1: Forgetting to Call `async_drop()`

The most common mistake. Will cause a panic at runtime.

```rust
// WRONG - panics with "Forgot to call async_drop on ..."
async fn bad() -> Result<()> {
    let resource = Resource::new();  // Returns AsyncDropGuard
    resource.do_work().await?;
    Ok(())
    // Panic here! async_drop was never called
}

// RIGHT
async fn good() -> Result<()> {
    let mut resource = Resource::new();
    resource.do_work().await?;
    resource.async_drop().await?;
    Ok(())
}

// BETTER - use the macro
async fn better() -> Result<()> {
    let resource = Resource::new();
    with_async_drop_2!(resource, {
        resource.do_work().await
    })
}
```

## Gotcha 2: Missing Cleanup on Early Returns

Every exit path needs cleanup, not just the happy path.

```rust
// WRONG - leaks on early return
async fn bad(mut resource: AsyncDropGuard<R>) -> Result<Data> {
    if !resource.is_valid() {
        return Err(Error::Invalid);  // Leaked!
    }

    let data = resource.fetch().await?;  // Leaked on error!
    resource.async_drop().await?;
    Ok(data)
}

// RIGHT - cleanup on all paths
async fn good(mut resource: AsyncDropGuard<R>) -> Result<Data> {
    if !resource.is_valid() {
        resource.async_drop().await?;
        return Err(Error::Invalid);
    }

    let result = resource.fetch().await;
    resource.async_drop().await?;
    result
}
```

## Gotcha 3: Allowing Direct Instantiation of AsyncDrop Types

AsyncDrop types must prevent callers from creating instances without an `AsyncDropGuard` wrapper. This applies to:
- Factory methods (must return `AsyncDropGuard<Self>`)
- Struct visibility (fields should be private)
- Enum variants (use newtype wrapper)

```rust
// WRONG - factory returns plain Self
impl MyType {
    pub fn new() -> Self {
        Self { /* ... */ }
    }
}

// WRONG - public fields allow direct construction
pub struct MyType {
    pub field: String,  // Caller can do: MyType { field: "".into() }
}

// WRONG - public enum variants allow direct construction
pub enum State {
    Ready(AsyncDropGuard<Resource>),  // Caller can do: State::Ready(...)
}

// RIGHT - private fields, factory returns guard
pub struct MyType {
    field: String,  // Private
}

impl MyType {
    pub fn new(field: String) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { field })
    }
}

// RIGHT - newtype wrapper with private inner enum
enum StateInner {
    Ready(AsyncDropGuard<Resource>),
}

pub struct State(StateInner);  // Inner is private

impl State {
    pub fn ready(resource: AsyncDropGuard<Resource>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self(StateInner::Ready(resource)))
    }
}
```

**Principle:** If callers can construct `Self` directly, they can bypass the `AsyncDropGuard` and cause panics or resource leaks.

## Gotcha 4: Types with Guard Members Not Implementing AsyncDrop

If a type holds `AsyncDropGuard` members, it must implement `AsyncDrop`.

```rust
// WRONG - inner guards never get async_drop called
pub struct Container {
    inner: AsyncDropGuard<Resource>,
}

impl Container {
    pub fn new() -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            inner: Resource::new(),
        })
    }
}
// Container's async_drop doesn't call inner.async_drop()!

// RIGHT
#[async_trait]
impl AsyncDrop for Container {
    type Error = <Resource as AsyncDrop>::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.inner.async_drop().await
    }
}
```

## Gotcha 5: Misusing `unsafe_into_inner_dont_drop()`

This method is for internal use within a type to unwrap `Self` from its guard. The type is still responsible for cleaning up its members.

```rust
// WRONG - using it to avoid cleanup
let guard = AsyncDropGuard::new(resource);
let inner = guard.unsafe_into_inner_dont_drop();
// inner is now unguarded - if you don't clean up members, they leak!

// WRONG - passing unwrapped value externally without cleanup
impl MyType {
    pub async fn bad(this: AsyncDropGuard<Self>) -> Result<()> {
        let inner = this.unsafe_into_inner_dont_drop();
        external_function(inner).await  // Who cleans up inner's members?
    }
}

// RIGHT - internal unwrapping, still clean up members
impl MyType {
    pub async fn good(this: AsyncDropGuard<Self>) -> Result<()> {
        let mut inner = this.unsafe_into_inner_dont_drop();
        let result = inner.member.do_work().await?;
        inner.member.async_drop().await?;  // Still our responsibility!
        Ok(result)
    }
}
```

## Gotcha 6: Using SyncDrop in Production

`SyncDrop` calls `async_drop()` synchronously in its `Drop` impl. This can deadlock.

```rust
// DANGEROUS - can deadlock on single-thread runtime
let guard = AsyncDropGuard::new(resource);
let sync = SyncDrop::new(guard);
drop(sync);  // Calls block_on(async_drop()) - may deadlock!
```

`SyncDrop` is intended for test utilities only. In production, always use async cleanup.

## Gotcha 7: Wrong Drop Order for Dependent Members

When members have dependencies, drop in correct order (usually reverse of construction).

```rust
pub struct System {
    cache: AsyncDropGuard<Cache>,      // Uses database
    database: AsyncDropGuard<Database>, // Independent
}

#[async_trait]
impl AsyncDrop for System {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        // WRONG order - database closes while cache still uses it
        // self.database.async_drop().await?;
        // self.cache.async_drop().await?;

        // RIGHT order - close cache first, then database
        self.cache.async_drop().await?;
        self.database.async_drop().await?;
        Ok(())
    }
}
```

For independent members, see Pattern 10 (Concurrent Cleanup) in patterns.md.

## Gotcha 8: Panics and AsyncDrop

Panics skip `async_drop()`. This is acceptable - panics are treated as unrecoverable.

```rust
async fn may_panic(mut resource: AsyncDropGuard<R>) -> Result<()> {
    resource.do_work().await?;

    // If this panics, resource.async_drop() is skipped
    // This is OK - panics are unrecoverable errors
    some_operation_that_may_panic();

    resource.async_drop().await?;
    Ok(())
}
```

Don't try to call `async_drop()` in panic handlers.

## Gotcha 9: Double async_drop

Calling `async_drop()` twice is harmless but wasteful - it returns `Ok(())` on second call.

```rust
let mut resource = Resource::new();
resource.async_drop().await?;  // Does cleanup
resource.async_drop().await?;  // No-op, returns Ok(())
```

Use `is_dropped()` to check if already dropped if needed:

```rust
if !resource.is_dropped() {
    resource.async_drop().await?;
}
```

## Gotcha 10: Holding Guards Across Await Points Without Cleanup

Long-lived guards in loops need careful handling.

```rust
// WRONG - accumulates guards without cleanup
async fn bad_loop() -> Result<()> {
    loop {
        let resource = Resource::new();
        resource.process().await?;
        // Guard dropped here - PANIC!
    }
}

// RIGHT - cleanup each iteration
async fn good_loop() -> Result<()> {
    loop {
        let mut resource = Resource::new();
        resource.process().await?;
        resource.async_drop().await?;
    }
}
```

## Gotcha 11: Clone vs AsyncDropArc

Regular `Clone` on `AsyncDropGuard` is not available. Use `AsyncDropArc` for shared ownership.

```rust
// WRONG - AsyncDropGuard doesn't implement Clone
let guard = AsyncDropGuard::new(resource);
let clone = guard.clone();  // Compile error!

// RIGHT - use AsyncDropArc for sharing
let shared = AsyncDropArc::new(AsyncDropGuard::new(resource));
let clone = AsyncDropArc::clone(&shared);
// Both must be async_dropped, last one does actual cleanup
```

## Gotcha 12: Forgetting `mut` Binding

`async_drop()` takes `&mut self`, so the guard must be mutable.

```rust
// WRONG - can't call async_drop on immutable binding
let resource = Resource::new();
resource.async_drop().await?;  // Error: cannot borrow as mutable

// RIGHT
let mut resource = Resource::new();
resource.async_drop().await?;
```

## Summary Checklist

Before submitting code with AsyncDrop:

- [ ] Every `AsyncDropGuard` has `async_drop()` called
- [ ] All error paths call `async_drop()` before returning
- [ ] All early returns call `async_drop()` first
- [ ] Factory methods return `AsyncDropGuard<Self>`
- [ ] Direct instantiation prevented (private fields, newtype wrappers for enums)
- [ ] Types with guard members implement `AsyncDrop`
- [ ] Guard bindings are `mut`
- [ ] `unsafe_into_inner_dont_drop()` only used internally, with member cleanup handled
- [ ] Drop order correct for dependent members (reverse of construction)
- [ ] Independent members dropped concurrently (Pattern 10)
- [ ] Using `with_async_drop_2!` where possible
