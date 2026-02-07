//! A string type for sensitive data (passwords, hex-encoded keys) that
//! is locked in memory and securely zeroed on drop.
//!
//! [`SensitiveString`] provides the same protections as
//! [`EncryptionKey`](crate::symmetric::EncryptionKey):
//! - Memory is locked to prevent swapping to disk (best-effort via `mlock`)
//! - Memory is securely zeroed when dropped
//!
//! # Example
//!
//! ```
//! use cryfs_crypto::sensitive_string::SensitiveString;
//!
//! let password = SensitiveString::new("my secret password".to_string());
//! assert_eq!(password.as_str(), "my secret password");
//! // Memory is zeroed when `password` goes out of scope
//! ```

use log::warn;
use std::fmt;
use std::ops::Deref;

/// A string type that protects sensitive data in memory.
///
/// The string data is:
/// - Locked in RAM to prevent the OS from swapping it to disk (best-effort)
/// - Securely zeroed when dropped using `sodiumoxide::utils::memzero`
///
/// This type is intended for storing passwords, hex-encoded key material,
/// and other sensitive string data that should not linger in memory.
///
/// Note that this is best-effort protection. There are scenarios (e.g.,
/// suspend to disk, prior copies from `rpassword`) where the data may
/// still end up on disk.
pub struct SensitiveString {
    data: Box<[u8]>,
    _lock_guard: Option<region::LockGuard>,
}

impl SensitiveString {
    /// Creates a new `SensitiveString` from the given `String`.
    ///
    /// The string data is moved into locked memory. The original `String`
    /// allocation is consumed but not explicitly zeroed (Rust's allocator
    /// may reuse the memory).
    ///
    /// # Arguments
    ///
    /// * `s` - The string to protect
    pub fn new(s: String) -> Self {
        let data: Box<[u8]> = s.into_bytes().into_boxed_slice();
        let lock_guard = region::lock(data.as_ptr(), data.len());
        let lock_guard = match lock_guard {
            Ok(lock_guard) => Some(lock_guard),
            Err(err) => {
                warn!(
                    "Couldn't lock RAM page for sensitive string data, \
                     which means it could get swapped to disk. \
                     This does not hinder any functionality. Error: {}",
                    err
                );
                None
            }
        };
        Self {
            data,
            _lock_guard: lock_guard,
        }
    }

    /// Returns the sensitive string data as a `&str`.
    pub fn as_str(&self) -> &str {
        // SAFETY: `data` was created from a valid String, so it's valid UTF-8.
        // We use debug_assert to catch any bugs in debug mode.
        debug_assert!(std::str::from_utf8(&self.data).is_ok());
        // This unwrap is safe because the data was created from a String
        std::str::from_utf8(&self.data).unwrap()
    }
}

impl Deref for SensitiveString {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl Drop for SensitiveString {
    fn drop(&mut self) {
        sodiumoxide::utils::memzero(&mut self.data);
    }
}

impl Clone for SensitiveString {
    fn clone(&self) -> Self {
        // Create a new locked copy of the data
        let data: Box<[u8]> = self.data.clone();
        let lock_guard = region::lock(data.as_ptr(), data.len());
        let lock_guard = match lock_guard {
            Ok(lock_guard) => Some(lock_guard),
            Err(err) => {
                warn!(
                    "Couldn't lock RAM page for sensitive string data, \
                     which means it could get swapped to disk. \
                     This does not hinder any functionality. Error: {}",
                    err
                );
                None
            }
        };
        Self {
            data,
            _lock_guard: lock_guard,
        }
    }
}

impl fmt::Debug for SensitiveString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SensitiveString(len={})", self.data.len())
    }
}

impl PartialEq for SensitiveString {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for SensitiveString {}

impl PartialEq<str> for SensitiveString {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for SensitiveString {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<SensitiveString> for str {
    fn eq(&self, other: &SensitiveString) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<SensitiveString> for &str {
    fn eq(&self, other: &SensitiveString) -> bool {
        *self == other.as_str()
    }
}

impl From<String> for SensitiveString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl serde::Serialize for SensitiveString {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for SensitiveString {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_access() {
        let s = SensitiveString::new("hello".to_string());
        assert_eq!(s.as_str(), "hello");
        assert_eq!(&*s, "hello");
    }

    #[test]
    fn clone_produces_equal_value() {
        let s = SensitiveString::new("secret".to_string());
        let cloned = s.clone();
        assert_eq!(s, cloned);
        assert_eq!(cloned.as_str(), "secret");
    }

    #[test]
    fn debug_does_not_leak_content() {
        let s = SensitiveString::new("password123".to_string());
        let debug = format!("{:?}", s);
        assert!(!debug.contains("password123"));
        assert!(debug.contains("SensitiveString"));
    }

    #[test]
    fn from_string() {
        let s: SensitiveString = "test".to_string().into();
        assert_eq!(s.as_str(), "test");
    }

    #[test]
    fn empty_string() {
        let s = SensitiveString::new(String::new());
        assert_eq!(s.as_str(), "");
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn equality() {
        let a = SensitiveString::new("same".to_string());
        let b = SensitiveString::new("same".to_string());
        let c = SensitiveString::new("different".to_string());
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn serde_roundtrip() {
        let original = SensitiveString::new("secret_value".to_string());
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "\"secret_value\"");
        let deserialized: SensitiveString = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }
}
