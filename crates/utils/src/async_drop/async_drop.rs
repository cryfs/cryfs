use std::fmt::Debug;
use std::future::Future;

/// Implement this trait to define an async drop behavior for your
/// type. See [AsyncDropGuard](super::AsyncDropGuard) for more details.
///
/// [AsyncDrop::async_drop_impl] takes `self` by value. Destructure `self` to move
/// [AsyncDropGuard](super::AsyncDropGuard) members out and call
/// [AsyncDropGuard::async_drop](super::AsyncDropGuard::async_drop) on them:
///
/// ```
/// use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};
///
/// #[derive(Debug)]
/// struct Connection;
///
/// impl AsyncDrop for Connection {
///     type Error = std::convert::Infallible;
///
///     async fn async_drop_impl(self) -> Result<(), Self::Error> {
///         // close the connection asynchronously
///         Ok(())
///     }
/// }
///
/// #[derive(Debug)]
/// struct Client {
///     connection: AsyncDropGuard<Connection>,
///     name: String,
/// }
///
/// impl AsyncDrop for Client {
///     type Error = std::convert::Infallible;
///
///     async fn async_drop_impl(self) -> Result<(), Self::Error> {
///         let Self { connection, name: _ } = self;
///         connection.async_drop().await
///     }
/// }
///
/// # futures::executor::block_on(async {
/// let client = AsyncDropGuard::new(Client {
///     connection: AsyncDropGuard::new(Connection),
///     name: "client".to_string(),
/// });
/// client.async_drop().await.unwrap();
/// # });
/// ```
///
/// A type that also implements [Drop] cannot be destructured. Store its
/// [AsyncDropGuard](super::AsyncDropGuard) members in an [Option] and use
/// [Option::take] in [AsyncDrop::async_drop_impl] instead.
pub trait AsyncDrop {
    type Error: Debug;

    /// Implement this to define drop behavior for your type.
    /// This will be called whenever
    /// [AsyncDropGuard::async_drop](super::AsyncDropGuard::async_drop)
    /// is executed while wrapping a value of the type implementing [AsyncDrop].
    ///
    /// The returned future must be [Send]. Implementations can use `async fn` syntax,
    /// the compiler then checks that the resulting future is [Send].
    ///
    /// If the implementing type also implements [Drop], then [Drop::drop]
    /// will be executed synchronously when `self` goes out of scope inside
    /// this method, i.e. for an `async fn` at the end of its body and after
    /// everything it awaited.
    ///
    /// [AsyncDrop::async_drop_impl] can return
    /// an error and that error will be propagated to the caller of
    /// [AsyncDropGuard::async_drop](super::AsyncDropGuard::async_drop).
    /// If such an error happens, [Drop::drop] still gets executed.
    fn async_drop_impl(self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
