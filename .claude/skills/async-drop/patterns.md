# AsyncDrop Patterns

Common patterns for implementing and using AsyncDrop in this codebase.

## Pattern 1: Simple AsyncDrop Implementation

For types that need async cleanup:

```rust
use cryfs_utils::{AsyncDrop, AsyncDropGuard};

pub struct MyResource {
    connection: Connection,
}

impl MyResource {
    // Factory returns AsyncDropGuard, not Self
    pub fn new(connection: Connection) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { connection })
    }
}

impl AsyncDrop for MyResource {
    type Error = anyhow::Error;

    // Takes `self` by value, no `#[async_trait]`
    async fn async_drop_impl(self) -> Result<(), Self::Error> {
        self.connection.close().await?;
        Ok(())
    }
}
```

## Pattern 2: Delegating to Member AsyncDrops

When a type contains `AsyncDropGuard` members, destructure `self` to move them out:

```rust
pub struct CompositeResource {
    database: AsyncDropGuard<Database>,
    cache: AsyncDropGuard<Cache>,
    name: String,
}

impl AsyncDrop for CompositeResource {
    type Error = anyhow::Error;

    async fn async_drop_impl(self) -> Result<(), Self::Error> {
        let Self { database, cache, name: _ } = self;
        // Drop members in appropriate order
        cache.async_drop().await?;
        database.async_drop().await?;
        Ok(())
    }
}
```

A type that also implements `Drop` cannot be destructured. Store its guard members
in an `Option` and `.take()` them in `async_drop_impl` instead.

## Pattern 3: Using `with_async_drop!` Macro

The preferred approach when it fits - automatically handles cleanup:

```rust
use cryfs_utils::with_async_drop;

async fn process_file(path: &Path) -> Result<Data> {
    let file = open_file(path).await?;  // Returns AsyncDropGuard<File>

    with_async_drop!(file, {
        // Use file here
        let data = file.read_all().await?;
        process(data).await
    })
    // file.async_drop() is called automatically
}
```

### Macro Variants

```rust
// Basic - propagates async_drop errors as-is
with_async_drop!(value, {
    // ... work ...
    Ok(result)
})

// With error mapping - converts async_drop errors
with_async_drop!(value, {
    // ... work ...
    Ok(result)
}, MyError::from)

// Infallible - for types with Error = Never
with_async_drop_infallible!(value, {
    // ... work ...
    result
})

// Several independent guards - all forms accept a list; the guards are dropped
// concurrently after the block (they must share the same error type)
with_async_drop!(source, dest, {
    // ... work with source and dest ...
    Ok(result)
})
```

Prefer the multi-guard form over nesting one `with_async_drop!` inside another: it
is flatter and drops the guards concurrently.

## Pattern 4: Keeping the Guard on Success, Dropping It on Error

When a function keeps the guard on its success path (e.g. moves it into a new object) but
must drop it on every error path, the macro doesn't fit. Use `async_drop_on_err`: it
drops the guard if the result is an error and returns that error, otherwise it hands the
value and the guard back. A failure to drop is logged; the original error is what gets returned.

```rust
async fn create_child(parent: AsyncDropGuard<Dir>) -> Result<Node> {
    let child_id = create_child_blob().await;
    // parent is dropped and the error returned if create_child_blob failed
    let (child_id, parent) = parent.async_drop_on_err(child_id).await?;

    let attrs = parent.add_entry(child_id).await;
    let (attrs, parent) = parent.async_drop_on_err(attrs).await?;

    Ok(Node::new(parent, child_id, attrs))  // parent moves into the node
}
```

Only write the cleanup by hand when neither the macro nor `async_drop_on_err` fits:

```rust
async fn complex_operation(mut resource: AsyncDropGuard<Resource>) -> Result<Output> {
    let result = resource.process().await;
    resource.async_drop().await?;
    result
}
```

## Pattern 5: Internal Unwrapping with `unsafe_into_inner_dont_drop()`

Use `unsafe_into_inner_dont_drop()` internally within a type to access the inner value when the type itself handles cleanup via its own `AsyncDrop` implementation.

```rust
pub struct Wrapper {
    inner: AsyncDropGuard<Resource>,
}

impl Wrapper {
    /// Consumes the wrapper to perform an operation on the inner resource.
    /// The Wrapper's AsyncDrop handles cleanup of the inner resource.
    pub async fn consume(this: AsyncDropGuard<Self>) -> Result<Output> {
        // Unwrap Self from its guard - we're inside our own impl
        let Self { mut inner } = this.unsafe_into_inner_dont_drop();

        // Now we can work with inner directly
        let result = inner.do_something().await?;

        // We MUST still clean up inner - our responsibility hasn't changed
        inner.async_drop().await?;

        Ok(result)
    }
}

impl AsyncDrop for Wrapper {
    type Error = anyhow::Error;

    async fn async_drop_impl(self) -> Result<(), Self::Error> {
        self.inner.async_drop().await?;
        Ok(())
    }
}
```

**Key point:** `unsafe_into_inner_dont_drop()` unwraps `Self` from its `AsyncDropGuard`, but the type's own `AsyncDrop` impl (or explicit cleanup in the consuming method) is still responsible for cleaning up members. This does NOT transfer responsibility elsewhere.

## Pattern 6: Conditional AsyncDrop with Newtype Wrapper

For types with multiple states (like enums), wrap in a newtype to prevent direct instantiation:

```rust
// Private enum - cannot be constructed outside this module
enum MaybeInitializedInner<T> {
    Uninitialized(Box<dyn FnOnce() -> AsyncDropGuard<T> + Send>),
    Initialized(AsyncDropGuard<T>),
}

// Public newtype - only way to create is via factory methods returning AsyncDropGuard
pub struct MaybeInitialized<T>(MaybeInitializedInner<T>);

impl<T> MaybeInitialized<T> {
    // Factory methods return AsyncDropGuard<Self>, never Self
    pub fn uninitialized(factory: impl FnOnce() -> AsyncDropGuard<T> + Send + 'static) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self(MaybeInitializedInner::Uninitialized(Box::new(factory))))
    }

    pub fn initialized(value: AsyncDropGuard<T>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self(MaybeInitializedInner::Initialized(value)))
    }
}

impl<T: AsyncDrop + Debug + Send> AsyncDrop for MaybeInitialized<T> {
    type Error = T::Error;

    async fn async_drop_impl(self) -> Result<(), Self::Error> {
        match self.0 {
            MaybeInitializedInner::Uninitialized(factory) => factory().async_drop().await,
            MaybeInitializedInner::Initialized(value) => value.async_drop().await,
        }
    }
}
```

**Key point:** The inner enum is private, so callers cannot construct `MaybeInitialized` directly - they must use factory methods that return `AsyncDropGuard<Self>`.

## Pattern 7: Passing Guards by Value

When passing `AsyncDropGuard<T>` by value, ownership and cleanup responsibility transfers:

```rust
// Caller is responsible for cleanup
async fn caller() -> Result<()> {
    let resource = create_resource();
    process_resource(resource).await?;  // Transfers ownership
    // No need to call async_drop - process_resource owns it now
    Ok(())
}

// Callee takes ownership, must clean up
async fn process_resource(mut resource: AsyncDropGuard<Resource>) -> Result<()> {
    resource.do_work().await?;
    resource.async_drop().await?;  // Callee's responsibility
    Ok(())
}
```

## Pattern 8: Returning Guards

When returning a guard, caller receives cleanup responsibility:

```rust
async fn create_and_configure() -> Result<AsyncDropGuard<Resource>> {
    let mut resource = Resource::new();  // Returns AsyncDropGuard
    resource.configure().await?;
    Ok(resource)  // Caller must async_drop
}

async fn use_it() -> Result<()> {
    let resource = create_and_configure().await?;
    resource.work().await?;
    resource.async_drop().await?;  // Our responsibility now
    Ok(())
}
```

## Pattern 9: Parallel Cleanup with AsyncDropHashMap

For collections of async-droppable values:

```rust
use cryfs_utils::AsyncDropHashMap;

let mut map: AsyncDropHashMap<String, Connection> = AsyncDropHashMap::new();
map.insert("db1".to_string(), Connection::new("db1").await?);
map.insert("db2".to_string(), Connection::new("db2").await?);

// All values are dropped in parallel
map.async_drop().await?;
```

## Pattern 10: Concurrent Cleanup for Independent Members

When a type has multiple independent members, drop them concurrently for better performance:

```rust
pub struct ConnectionPool {
    conn_a: AsyncDropGuard<Connection>,
    conn_b: AsyncDropGuard<Connection>,
    conn_c: AsyncDropGuard<Connection>,
}

impl AsyncDrop for ConnectionPool {
    type Error = anyhow::Error;

    async fn async_drop_impl(self) -> Result<(), Self::Error> {
        let Self { conn_a, conn_b, conn_c } = self;
        // GOOD - concurrent drop for independent resources
        async_drop_all((conn_a, conn_b, conn_c)).await
    }
}
```

`async_drop_all` takes a tuple of guards (or `Option<AsyncDropGuard<_>>`s) sharing one error
type, drops them concurrently, waits for all of them even if some fail, and returns the first
error (the others are logged). Use it whenever members don't depend on each other.

## Pattern 11: Shared Ownership with AsyncDropArc

When multiple owners need access:

```rust
use cryfs_utils::AsyncDropArc;

let shared = AsyncDropArc::new(AsyncDropGuard::new(resource));
let clone1 = AsyncDropArc::clone(&shared);
let clone2 = AsyncDropArc::clone(&shared);

// All clones must be dropped
clone1.async_drop().await?;
clone2.async_drop().await?;
shared.async_drop().await?;  // Last one does actual cleanup
```

## Pattern 12: Error Type Selection

Choose error types based on context:

```rust
// Specific error for library types
impl AsyncDrop for DatabaseConnection {
    type Error = DatabaseError;  // Specific, detailed
    // ...
}

// Anyhow for application-level types
impl AsyncDrop for AppResource {
    type Error = anyhow::Error;  // Flexible
    // ...
}

// Never for infallible cleanup
impl AsyncDrop for SimpleBuffer {
    type Error = std::convert::Infallible;

    async fn async_drop_impl(self) -> Result<(), Self::Error> {
        drop(self.data);  // Can't fail
        Ok(())
    }
}
```

## Anti-Pattern: Forgetting Cleanup in Error Paths

```rust
// WRONG - leaks resource on error
async fn bad_example(mut resource: AsyncDropGuard<R>) -> Result<()> {
    resource.step1().await?;  // If this fails, resource leaks!
    resource.async_drop().await?;
    Ok(())
}

// RIGHT - cleanup on all paths
async fn good_example(mut resource: AsyncDropGuard<R>) -> Result<()> {
    let result = resource.step1().await;
    resource.async_drop().await?;
    result?;
    Ok(())
}

// BETTER - use the macro
async fn best_example(resource: AsyncDropGuard<R>) -> Result<()> {
    with_async_drop!(resource, {
        resource.step1().await
    })
}
```
