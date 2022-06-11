use std::fmt::Debug;
use thiserror::Error;

/// Errors that can be thrown by [LockPool::try_lock](super::LockPool::try_lock).
#[derive(Error, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TryLockError {
    /// The lock could not be acquired at this time because the operation would otherwise block
    #[error(
        "The lock could not be acquired at this time because the operation would otherwise block"
    )]
    WouldBlock,
}
