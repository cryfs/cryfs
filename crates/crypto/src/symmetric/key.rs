use anyhow::{Result, ensure};
use log::warn;

// TODO Separate InfallibleUnwrap from the lockable crate and don't depend on lockable from this crate
use lockable::InfallibleUnwrap;

// TODO The 'secrets' crate looks interesting as a replacement to 'region',
// but the dependency didn't compile for me.

/// An encryption key for a cipher. The key is stored in protected memory, i.e.
/// it shouldn't be swapped to disk and will be automatically zeroed on destruction.
/// Note that this is only a best-effort and not guaranteed. There's still scenarios
/// (say when the PC is suspended to disk) where the key will end up on the disk.
pub struct EncryptionKey {
    key_data: Box<[u8]>,
    _lock_guard: Option<region::LockGuard>,
}

impl EncryptionKey {
    pub fn new<E>(
        num_bytes: usize,
        init: impl FnOnce(&mut [u8]) -> Result<(), E>,
    ) -> Result<Self, E> {
        let mut key_data: Box<[u8]> = vec![0u8; num_bytes].into_boxed_slice();
        let lock_guard = region::lock(key_data.as_ptr(), key_data.len());
        let lock_guard = match lock_guard {
            Ok(lock_guard) => Some(lock_guard),
            Err(err) => {
                warn!(
                    "Couldn't protect the RAM page storing the encryption key, which means it could get swapped to the disk if your operating system chooses to. This does not hinder any functionality though. Error: {}",
                    err
                );
                None
            }
        };
        // TODO mprotect would be nice too
        init(&mut key_data)?;
        Ok(Self {
            key_data,
            _lock_guard: lock_guard,
        })
    }

    /// Create key data from a hex string. This can be super helpful for test cases
    /// but it circumvents the protection because the data exists somewhere else
    /// before creating the EncryptionKey object. So we're making sure it's actually
    /// only available to test cases using cfg(test).
    // TODO #[cfg(test)]
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        ensure!(
            hex_str.len().is_multiple_of(2),
            "Hex string must have an even length"
        );
        let num_bytes = hex_str.len() / 2;
        Self::new(num_bytes, |data| {
            hex::decode_to_slice(hex_str, data)?;
            Ok(())
        })
    }

    /// Create a hex string with the key data. This can be super helpful for test cases
    /// but it circumvents the protection because the data gets copied to an unprotected
    /// string. So we're making sure it's actually only available to test cases using cfg(test).
    /// TODO Make this actually true and add a #[cfg(test)] here
    pub fn to_hex(&self) -> String {
        hex::encode_upper(&self.key_data)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.key_data
    }

    pub fn num_bytes(&self) -> usize {
        self.key_data.len()
    }

    /// Copies the first `num_bytes` bytes of the [EncryptionKey] into a new [EncryptionKey]
    pub fn take_bytes(&self, num_bytes: usize) -> EncryptionKey {
        Self::new(num_bytes, |data| {
            data.copy_from_slice(&self.key_data[..num_bytes]);
            Ok(())
        })
        .infallible_unwrap()
    }

    /// Skips the first `num_bytes` bytes of the [EncryptionKey] and returns a new [EncryptionKey] with the remaining bytes.
    pub fn skip_bytes(&self, num_bytes: usize) -> EncryptionKey {
        Self::new(self.key_data.len() - num_bytes, |data| {
            data.copy_from_slice(&self.key_data[num_bytes..]);
            Ok(())
        })
        .infallible_unwrap()
    }
}

impl std::fmt::Debug for EncryptionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EncryptionKey(len={})", self.key_data.len())
    }
}

impl Drop for EncryptionKey {
    fn drop(&mut self) {
        sodiumoxide::utils::memzero(&mut self.key_data);
    }
}

impl PartialEq for EncryptionKey {
    fn eq(&self, other: &Self) -> bool {
        self.key_data == other.key_data
    }
}

impl Eq for EncryptionKey {}
