mod async_drop;
pub use async_drop::AsyncDrop;

mod async_drop_guard;
pub use async_drop_guard::AsyncDropGuard;

mod async_drop_arc;
pub use async_drop_arc::AsyncDropArc;

#[cfg(test)]
mod sync_drop;
#[cfg(test)]
pub use sync_drop::SyncDrop;
