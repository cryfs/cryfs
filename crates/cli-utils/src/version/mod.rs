mod version;

#[cfg(feature = "check_for_updates")]
mod http_client;

#[cfg(all(test, feature = "check_for_updates"))]
pub use http_client::FakeHttpClient;
#[cfg(feature = "check_for_updates")]
pub use http_client::{HttpClient, ReqwestHttpClient};
pub use version::show_version;
