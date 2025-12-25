# AsyncDrop Patterns

Common patterns for implementing and using AsyncDrop in this codebase.

## Pattern 1: Simple AsyncDrop Implementation

For types that need async cleanup:

```rust
use async_trait::async_trait;
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

#[async_trait]
impl AsyncDrop for MyResource {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.connection.close().await?;
        Ok(())
    }
}
```

## Pattern 2: Delegating to Member AsyncDrops

When a type contains `AsyncDropGuard` members:

```rust
pub struct CompositeResource {
    database: AsyncDropGuard<Database>,
    cache: AsyncDropGuard<Cache>,
}

#[async_trait]
impl AsyncDrop for CompositeResource {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        // Drop members in appropriate order
        self.cache.async_drop().await?;
        self.database.async_drop().await?;
        Ok(())
    }
}
```

## Pattern 3: Using `with_async_drop_2!` Macro

The preferred approach when it fits - automatically handles cleanup:

```rust
use cryfs_utils::with_async_drop_2;

async fn process_file(path: &Path) -> Result<Data> {
    let file = open_file(path).await?;  // Returns AsyncDropGuard<File>

    with_async_drop_2!(file, {
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
with_async_drop_2!(value, {
    // ... work ...
    Ok(result)
})

// With error mapping - converts async_drop errors
with_async_drop_2!(value, {
    // ... work ...
    Ok(result)
}, MyError::from)

// Infallible - for types with Error = Never
with_async_drop_2_infallible!(value, {
    // ... work ...
    result
})
```

## Pattern 4: Manual Cleanup on All Exit Paths

When the macro doesn't fit, manually ensure cleanup on every path:

```rust
async fn complex_operation(resource: AsyncDropGuard<Resource>) -> Result<Output> {
    let mut resource = resource;

    // Early return path 1
    if !resource.is_valid() {
        resource.async_drop().await?;
        return Err(Error::Invalid);
    }

    // Main work
    let result = match resource.process().await {
        Ok(data) => data,
        Err(e) => {
            resource.async_drop().await?;  // Don't forget!
            return Err(e.into());
        }
    };

    // Success path
    resource.async_drop().await?;
    Ok(result)
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
        let mut this = this.unsafe_into_inner_dont_drop();

        // Now we can work with this.inner directly
        let result = this.inner.do_something().await?;

        // We MUST still clean up inner - our responsibility hasn't changed
        this.inner.async_drop().await?;

        Ok(result)
    }
}

#[async_trait]
impl AsyncDrop for Wrapper {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
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
    Uninitialized(Option<Box<dyn FnOnce() -> AsyncDropGuard<T>>>),
    Initialized(AsyncDropGuard<T>),
}

// Public newtype - only way to create is via factory methods returning AsyncDropGuard
pub struct MaybeInitialized<T>(MaybeInitializedInner<T>);

impl<T> MaybeInitialized<T> {
    // Factory methods return AsyncDropGuard<Self>, never Self
    pub fn uninitialized(factory: impl FnOnce() -> AsyncDropGuard<T> + 'static) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self(MaybeInitializedInner::Uninitialized(Some(Box::new(factory)))))
    }

    pub fn initialized(value: AsyncDropGuard<T>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self(MaybeInitializedInner::Initialized(value)))
    }
}

#[async_trait]
impl<T: AsyncDrop + Debug + Send> AsyncDrop for MaybeInitialized<T> {
    type Error = T::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        match &mut self.0 {
            MaybeInitializedInner::Uninitialized(factory) => {
                if let Some(factory) = factory.take() {
                    factory().async_drop().await?;
                }
            }
            MaybeInitializedInner::Initialized(value) => {
                value.async_drop().await?;
            }
        }
        Ok(())
    }
}
```

**Key point:** The inner enum is private, so callers cannot construct `MaybeInitialized` directly - they must use factory methods that return `AsyncDropGuard<Self>`.

## Pattern 7: Passing Guards by Value

When passing `AsyncDropGuard<T>` by value, ownership and cleanup responsibility transfers:

```rust
// Caller is responsible for cleanup
async fn caller() -> Result<()> {
    let mut resource = create_resource();
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
    let mut resource = create_and_configure().await?;
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

#[async_trait]
impl AsyncDrop for ConnectionPool {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        // GOOD - concurrent drop for independent resources
        let (a, b, c) = tokio::join!(
            self.conn_a.async_drop(),
            self.conn_b.async_drop(),
            self.conn_c.async_drop()
        );
        a?;
        b?;
        c?;
        Ok(())
    }
}
```

Use `tokio::join!` to run async_drop calls concurrently when members don't depend on each other.

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
#[async_trait]
impl AsyncDrop for DatabaseConnection {
    type Error = DatabaseError;  // Specific, detailed
    // ...
}

// Anyhow for application-level types
#[async_trait]
impl AsyncDrop for AppResource {
    type Error = anyhow::Error;  // Flexible
    // ...
}

// Never for infallible cleanup
#[async_trait]
impl AsyncDrop for SimpleBuffer {
    type Error = std::convert::Infallible;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.data.clear();  // Can't fail
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
    with_async_drop_2!(resource, {
        resource.step1().await
    })
}
```
