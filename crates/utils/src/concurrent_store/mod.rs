// TODO Move to a separate crate

mod entry;
mod guard;
mod store;

pub use guard::LoadedEntryGuard;
pub use store::{ConcurrentStore, RequestImmediateDropResult};
