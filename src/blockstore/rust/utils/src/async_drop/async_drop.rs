use async_trait::async_trait;
use std::fmt::Debug;

/// Implement this trait to define an async drop behavior for your
/// type. See [AsyncDropGuard] for more details.
#[async_trait]
pub trait AsyncDrop {
    type Error: Debug;

    /// Implement this to define drop behavior for your type.
    /// This will be called whenever [AsyncDropGuard::async_drop] is executed
    /// while wrapping a value of the type implementing [AsyncDrop].
    ///
    /// If the implementing type also implements [Drop], then [Drop::drop]
    /// will be executed synchronously and after [AsyncDrop::async_drop_impl].
    ///
    /// [AsyncDrop::async_drop_impl] can return an error and that error
    /// will be propagated to the caller of [AsyncDropGuard::async_drop_impl].
    /// If such an error happens, [Drop::drop] still gets executed.
    async fn async_drop_impl(&mut self) -> Result<(), Self::Error>;
}
