# AsyncDrop Helper Types

Wrapper types and utilities for common AsyncDrop scenarios.

## AsyncDropGuard<T>

The core wrapper that enforces async cleanup.

```rust
pub struct AsyncDropGuard<T: Debug>(Option<T>);
```

### Key Methods

| Method | Description |
|--------|-------------|
| `new(v: T)` | Wrap a value |
| `async_drop(&mut self)` | Perform async cleanup (required!) |
| `is_dropped(&self)` | Check if already dropped |
| `unsafe_into_inner_dont_drop(self)` | Extract inner, bypassing cleanup |
| `map_unsafe<U>(self, f)` | Transform inner type |

### Behavior

- Implements `Deref` and `DerefMut` for transparent access
- Has `#[must_use]` attribute
- Panics in `Drop` if `async_drop()` was not called

```rust
let mut guard = AsyncDropGuard::new(value);
guard.method();  // Deref to inner
guard.async_drop().await?;
```

---

## AsyncDropArc<T>

Reference-counted sharing of async-droppable values.

```rust
pub struct AsyncDropArc<T: AsyncDrop + Debug + Send> {
    v: Option<Arc<AsyncDropGuard<T>>>,
}
```

### Use Case

Multiple owners need access to the same resource. Only the last owner's `async_drop()` performs actual cleanup.

### Usage

```rust
let shared = AsyncDropArc::new(AsyncDropGuard::new(resource));

// Clone for multiple owners
let clone1 = AsyncDropArc::clone(&shared);
let clone2 = AsyncDropArc::clone(&shared);

// All must be dropped
clone1.async_drop().await?;  // No-op (not last)
clone2.async_drop().await?;  // No-op (not last)
shared.async_drop().await?;  // Actual cleanup (last Arc)
```

### Key Methods

| Method | Description |
|--------|-------------|
| `new(guard)` | Wrap an AsyncDropGuard |
| `clone(&self)` | Create another reference |
| `async_drop(&mut self)` | Drop this reference (cleanup on last) |

---

## AsyncDropTokioMutex<T>

Async mutex holding an async-droppable value.

```rust
pub struct AsyncDropTokioMutex<T: AsyncDrop + Debug + Send> {
    v: Option<Mutex<AsyncDropGuard<T>>>,
}
```

### Use Case

Safe concurrent access to an `AsyncDropGuard` value via async mutex.

### Usage

```rust
let mutex = AsyncDropTokioMutex::new(AsyncDropGuard::new(resource));

// Access via lock
{
    let mut guard = mutex.lock().await;
    guard.do_work().await?;
}

// Cleanup
mutex.async_drop().await?;
```

---

## AsyncDropHashMap<K, V>

HashMap that properly cleans up async-droppable values.

```rust
pub struct AsyncDropHashMap<K, V>
where
    K: PartialEq + Eq + Hash + Debug + Send,
    V: AsyncDrop + Send + Debug,
{
    map: HashMap<K, AsyncDropGuard<V>>,
}
```

### Use Case

Collections of resources that all need async cleanup.

### Usage

```rust
let mut map = AsyncDropHashMap::new();
map.insert("conn1".to_string(), Connection::new("db1").await?);
map.insert("conn2".to_string(), Connection::new("db2").await?);

// Access entries
if let Some(conn) = map.get_mut("conn1") {
    conn.query().await?;
}

// Cleanup all entries (in parallel!)
map.async_drop().await?;
```

### Key Methods

| Method | Description |
|--------|-------------|
| `new()` | Create empty map |
| `insert(k, v)` | Add entry (v is `AsyncDropGuard<V>`) |
| `get(&k)` | Get reference to value |
| `get_mut(&k)` | Get mutable reference |
| `remove(&k)` | Remove and return entry |
| `async_drop()` | Drop all values in parallel |

---

## AsyncDropResult<T, E>

Wraps `Result<AsyncDropGuard<T>, E>`.

```rust
pub struct AsyncDropResult<T, E>
where
    T: Debug + AsyncDrop + Send,
    E: Debug + Send,
{
    v: Result<AsyncDropGuard<T>, E>,
}
```

### Use Case

When you have a Result that might contain an async-droppable value.

### Behavior

- `async_drop()` only acts on `Ok` variant
- `Err` variant is left untouched

```rust
let result: AsyncDropResult<Resource, Error> = try_create_resource().await;

// Cleanup handles Ok case, ignores Err
result.async_drop().await?;
```

---

## SyncDrop<T>

Wrapper that calls `async_drop()` synchronously in its `Drop` impl.

```rust
pub struct SyncDrop<T: Debug + AsyncDrop>(Option<AsyncDropGuard<T>>);
```

### WARNING: Can Deadlock!

This uses `block_on()` internally, which can deadlock if:
- Running on a single-threaded runtime
- Called from within an async context

### Use Case

**Test utilities only.** When you need synchronous Drop semantics in tests.

```rust
// In tests only!
#[test]
fn test_something() {
    let resource = SyncDrop::new(AsyncDropGuard::new(create_resource()));
    // Use resource...
    // Automatically cleaned up on drop
}
```

DO NOT use SyncDrop outside of tests. It blocks the current thread until
async_drop is complete and will cause bad performance.

---

## Utility Functions

### `with_async_drop()`

Function version of the macro for more complex scenarios.

```rust
pub async fn with_async_drop<T, R, E, F>(
    mut value: AsyncDropGuard<T>,
    f: impl FnOnce(&mut T) -> F,
) -> Result<R, E>
where
    T: AsyncDrop + Debug,
    E: From<<T as AsyncDrop>::Error>,
    F: Future<Output = Result<R, E>>,
```

### `flatten_async_drop()`

Combines two Results of AsyncDropGuards.

```rust
pub async fn flatten_async_drop<E, T, E1, U, E2>(
    first: Result<AsyncDropGuard<T>, E1>,
    second: Result<AsyncDropGuard<U>, E2>,
) -> Result<(AsyncDropGuard<T>, AsyncDropGuard<U>), E>
```

Returns tuple of both guards if both are Ok. On error, properly cleans up any successful guard before returning error.

---

## AsyncDropShared<O, Fut>

Advanced: Shares a future that returns an `AsyncDropGuard<O>`.

### Use Case

Multiple tasks await the same resource creation. First to poll drives the future; all get access to the result via `AsyncDropArc`.

```rust
let shared = AsyncDropShared::new(async { create_expensive_resource().await });

// Multiple tasks can await
let clone1 = shared.clone();
let clone2 = shared.clone();

// All get access to the same resource
let result1: AsyncDropArc<Resource> = clone1.await?;
let result2: AsyncDropArc<Resource> = clone2.await?;
```

---

## Choosing the Right Helper

| Scenario | Use |
|----------|-----|
| Single owner, needs async cleanup | `AsyncDropGuard<T>` |
| Multiple owners, shared access | `AsyncDropArc<T>` |
| Concurrent mutable access | `AsyncDropTokioMutex<T>` |
| Collection of resources | `AsyncDropHashMap<K, V>` |
| Result that might need cleanup | `AsyncDropResult<T, E>` |
| Shared future result | `AsyncDropShared<O, Fut>` |
| Test utilities only | `SyncDrop<T>` |
