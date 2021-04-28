use anyhow::Result;
use generic_array::{ArrayLength, GenericArray};
use log::warn;

// TODO The 'secrets' crate looks interesting as a replacement to 'region',
// but the dependency didn't compile for me.

/// An encryption key for a cipher. The key is stored in protected memory, i.e.
/// it shouldn't be swapped to disk and will be automatically zeroed on destruction.
/// Note that this is only a best-effort and not guaranteed. There's still scenarios
/// (say when the PC is suspended to disk) where the key will end up on the disk.
pub struct EncryptionKey<KeySize: ArrayLength<u8>> {
    key_data: Box<GenericArray<u8, KeySize>>,
    _lock_guard: Option<region::LockGuard>,
}

impl<KeySize: ArrayLength<u8>> EncryptionKey<KeySize> {
    pub fn new(init: impl FnOnce(&mut [u8]) -> Result<()>) -> Result<Self> {
        let mut key_data = Box::new(GenericArray::default());
        let lock_guard = region::lock(key_data.as_slice().as_ptr(), key_data.as_slice().len());
        let lock_guard = match lock_guard {
            Ok(lock_guard) => Some(lock_guard),
            Err(err) => {
                warn!("Couldn't protect the RAM page storing the encryption key, which means it could get swapped to the disk if your operating system chooses to. This does not hinder any functionality though. Error: {}", err);
                None
            }
        };
        // TOTO mprotect would be nice too
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
        Self::new(|data| Ok(data.copy_from_slice(&hex::decode(hex_str)?)))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.key_data
    }
}

impl<KeySize: ArrayLength<u8>> std::fmt::Debug for EncryptionKey<KeySize> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EncryptionKey<{}>(****)", KeySize::USIZE)
    }
}

impl<KeySize: ArrayLength<u8>> Drop for EncryptionKey<KeySize> {
    fn drop(&mut self) {
        sodiumoxide::utils::memzero(&mut self.key_data);
    }
}
