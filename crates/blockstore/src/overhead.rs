use byte_unit::Byte;
use derive_more::{Display, Error};
use std::ops::Add;

/// Represents the overhead of a block store, i.e. how many overhead bytes are stored but not usable by call sits.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Overhead {
    overhead: Byte,
}

impl Add for Overhead {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            overhead: self.overhead.add(other.overhead).unwrap(),
        }
    }
}

impl Overhead {
    pub fn new(overhead: Byte) -> Self {
        Self { overhead }
    }

    pub fn usable_block_size_from_physical_block_size(
        &self,
        physical_block_size: Byte,
    ) -> Result<Byte, InvalidBlockSizeError> {
        physical_block_size.subtract(self.overhead).ok_or_else(|| {
            InvalidBlockSizeError::new(format!(
                "Physical block size {} is smaller than overhead {}",
                physical_block_size, self.overhead
            ))
        })
    }

    pub fn physical_block_size_from_usable_block_size(&self, usable_block_size: Byte) -> Byte {
        usable_block_size.add(self.overhead).unwrap()
    }
}

#[derive(Error, Display, Debug, PartialEq, Eq)]
#[display("Invalid block size: {message}")]
pub struct InvalidBlockSizeError {
    message: String,
}
impl InvalidBlockSizeError {
    pub fn new(message: String) -> Self {
        Self { message: message }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usable_block_size_from_physical_block_size() {
        assert_eq!(
            Ok(Byte::from_u64(90)),
            Overhead::new(Byte::from_u64(10))
                .usable_block_size_from_physical_block_size(Byte::from_u64(100))
        );
    }

    #[test]
    fn usable_block_size_from_physical_block_size_zero() {
        assert_eq!(
            Ok(Byte::from_u64(0)),
            Overhead::new(Byte::from_u64(10))
                .usable_block_size_from_physical_block_size(Byte::from_u64(10))
        );
    }

    #[test]
    fn usable_block_size_from_physical_block_size_error() {
        assert!(
            Overhead::new(Byte::from_u64(10))
                .usable_block_size_from_physical_block_size(Byte::from_u64(9))
                .is_err()
        );
    }

    #[test]
    fn physical_block_size_from_usable_block_size() {
        assert_eq!(
            Byte::from_u64(110),
            Overhead::new(Byte::from_u64(10))
                .physical_block_size_from_usable_block_size(Byte::from_u64(100))
        );
    }

    #[test]
    fn physical_to_usable_to_physical() {
        let overhead = Overhead::new(Byte::from_u64(10));

        let physical = Byte::from_u64(123);
        assert_eq!(
            physical,
            overhead.physical_block_size_from_usable_block_size(
                overhead
                    .usable_block_size_from_physical_block_size(physical)
                    .unwrap()
            )
        );
    }

    #[test]
    fn usable_to_physical_to_usable() {
        let overhead = Overhead::new(Byte::from_u64(10));

        let usable = Byte::from_u64(123);
        assert_eq!(
            usable,
            overhead
                .usable_block_size_from_physical_block_size(
                    overhead.physical_block_size_from_usable_block_size(usable)
                )
                .unwrap()
        );
    }
}
