//! Zero-initialized data wrapper.
//!
//! This module provides [`ZeroedData`], which wraps a data buffer and guarantees
//! that its contents are zeroed. This is useful for security-sensitive contexts
//! where uninitialized data could be a risk.

use super::Data;

/// A wrapper around a data object with the invariant that the data object is zeroed out.
///
/// This type guarantees that the contained data is all zeros, either by:
/// - Creating a new zero-filled buffer with [`ZeroedData::new`]
/// - Zeroing an existing buffer with [`ZeroedData::fill_with_zeroes`]
pub struct ZeroedData<D: AsRef<[u8]> + AsMut<[u8]>> {
    data: D,
}

impl ZeroedData<Data> {
    /// Creates a new `ZeroedData` with the specified length, filled with zeros.
    pub fn new(len: usize) -> Self {
        Self {
            data: Data::from(vec![0; len]),
        }
    }
}

impl<D: AsRef<[u8]> + AsMut<[u8]>> ZeroedData<D> {
    /// Creates a `ZeroedData` from an existing buffer, filling it with zeros first.
    pub fn fill_with_zeroes(mut data: D) -> Self {
        data.as_mut().fill(0);
        Self { data }
    }

    /// Consumes the wrapper and returns the inner data buffer.
    pub fn into_inner(self) -> D {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_zeroed_data() {
        let zeroed = ZeroedData::new(10);
        let data = zeroed.into_inner();
        assert_eq!(data.len(), 10);
        assert!(data.as_ref().iter().all(|&b| b == 0));
    }

    #[test]
    fn test_new_empty() {
        let zeroed = ZeroedData::new(0);
        let data = zeroed.into_inner();
        assert_eq!(data.len(), 0);
    }

    #[test]
    fn test_fill_with_zeroes() {
        let original = vec![1u8, 2, 3, 4, 5];
        let zeroed = ZeroedData::fill_with_zeroes(original);
        let data = zeroed.into_inner();
        assert_eq!(data.len(), 5);
        assert!(data.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_fill_with_zeroes_already_zeroed() {
        let original = vec![0u8; 5];
        let zeroed = ZeroedData::fill_with_zeroes(original);
        let data = zeroed.into_inner();
        assert_eq!(vec![0u8; 5], data);
    }
}
