use anyhow::Result;
use std::time::Duration;

/// A trait for running HTTP requests, that can be mocked for tests
pub trait HttpClient {
    fn get(&self, url: &str, timeout: Duration) -> Result<String>;
}

/// A real implementation of the `HttpClient` trait using the `reqwest` crate
pub struct ReqwestHttpClient;
impl HttpClient for ReqwestHttpClient {
    fn get(&self, url: &str, timeout: Duration) -> Result<String> {
        use reqwest::blocking::Client;
        let client = Client::builder().build()?;
        let response = client.get(url).timeout(timeout).send()?.text()?;
        Ok(response)
    }
}

#[cfg(test)]
mod fake_http_client {
    use super::*;
    use anyhow::anyhow;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;

    /// A fake implementation of the `HttpClient` trait that can be used for testing
    pub struct FakeHttpClient {
        pub websites: HashMap<String, String>,
        pub request_count: Arc<AtomicUsize>,
    }

    impl FakeHttpClient {
        pub fn new() -> Self {
            Self {
                websites: HashMap::new(),
                request_count: Arc::new(AtomicUsize::new(0)),
            }
        }

        pub fn request_counter(&self) -> Arc<AtomicUsize> {
            Arc::clone(&self.request_count)
        }

        /// Add a website to the fake client. Any future requests to this URL will return the given content
        pub fn add_website(&mut self, url: String, content: String) {
            self.websites.insert(url, content);
        }
    }

    impl HttpClient for FakeHttpClient {
        fn get(&self, url: &str, _timeout: Duration) -> Result<String> {
            self.request_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.websites
                .get(url)
                .cloned()
                .ok_or_else(|| anyhow!("URL not found: {}", url))
        }
    }
}
#[cfg(test)]
pub use fake_http_client::FakeHttpClient;

#[cfg(test)]
mod tests {
    use super::*;

    mod fake_http_client {
        use super::*;

        #[test]
        fn no_websites_known() {
            let client = FakeHttpClient::new();
            assert_eq!(
                "URL not found: https://example.com",
                client
                    .get("https://example.com", Duration::from_secs(10))
                    .unwrap_err()
                    .to_string()
            );
        }

        #[test]
        fn unknown_website() {
            let mut client = FakeHttpClient::new();
            client.add_website("https://existing.com".to_string(), "content".to_string());
            assert_eq!(
                "URL not found: https://notexisting.com",
                client
                    .get("https://notexisting.com", Duration::from_secs(10))
                    .unwrap_err()
                    .to_string()
            );
        }

        #[test]
        fn website_existing() {
            let mut client = FakeHttpClient::new();
            client.add_website(
                "https://example.com".to_string(),
                "Hello, world!".to_string(),
            );
            assert_eq!(
                client
                    .get("https://example.com", Duration::from_secs(10))
                    .unwrap(),
                "Hello, world!"
            );
        }

        #[test]
        fn two_websites_existing() {
            let mut client = FakeHttpClient::new();
            client.add_website("https://first.com".to_string(), "First website".to_string());
            client.add_website(
                "https://second.com".to_string(),
                "Second website".to_string(),
            );
            assert_eq!(
                client
                    .get("https://first.com", Duration::from_secs(10))
                    .unwrap(),
                "First website"
            );
            assert_eq!(
                client
                    .get("https://second.com", Duration::from_secs(10))
                    .unwrap(),
                "Second website"
            );
            assert_eq!(
                "URL not found: https://notexisting.com",
                client
                    .get("https://notexisting.com", Duration::from_secs(10))
                    .unwrap_err()
                    .to_string()
            );
        }

        #[test]
        fn overwriting() {
            let mut client = FakeHttpClient::new();
            client.add_website("http://existing.com".to_string(), "content".to_string());
            client.add_website("http://existing.com".to_string(), "new_content".to_string());
            assert_eq!(
                client
                    .get("http://existing.com", Duration::from_secs(10))
                    .unwrap(),
                "new_content"
            );
        }

        #[test]
        fn request_counter() {
            let client = FakeHttpClient::new();
            let request_counter = client.request_counter();
            assert_eq!(0, request_counter.load(std::sync::atomic::Ordering::SeqCst));
            client
                .get("http://example.com", Duration::from_secs(10))
                .unwrap_err();
            assert_eq!(1, request_counter.load(std::sync::atomic::Ordering::SeqCst));
            client
                .get("http://example.com", Duration::from_secs(10))
                .unwrap_err();
            assert_eq!(2, request_counter.load(std::sync::atomic::Ordering::SeqCst));
        }
    }

    mod reqwest_http_client {
        use super::*;

        #[test]
        fn test_get_invalid_protocol() {
            let client = ReqwestHttpClient;
            assert!(
                client
                    .get("invalid://example.com", Duration::from_secs(10))
                    .is_err()
            );
        }

        #[test]
        fn test_get_invalid_tld() {
            let client = ReqwestHttpClient;
            assert!(
                client
                    .get("http://example.invalidtld", Duration::from_secs(10))
                    .is_err()
            );
        }

        #[test]
        fn test_get_invalid_domain() {
            let client = ReqwestHttpClient;
            assert!(
                client
                    .get(
                        "http://this_is_a_not_existing_domain.com",
                        Duration::from_secs(10)
                    )
                    .is_err()
            );
        }

        #[test]
        fn test_get_valid_http() {
            let client = ReqwestHttpClient;
            let response = client
                .get("http://example.com", Duration::from_secs(10))
                .unwrap();
            assert!(response.contains("Example Domain"));
        }

        #[test]
        fn test_get_valid_https() {
            let client = ReqwestHttpClient;
            let response = client
                .get("https://example.com", Duration::from_secs(10))
                .unwrap();
            assert!(response.contains("Example Domain"));
        }
    }
}
