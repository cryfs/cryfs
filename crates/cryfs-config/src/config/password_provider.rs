use anyhow::Result;
use cryfs_crypto::sensitive_string::SensitiveString;

/// Trait for providing passwords for filesystem encryption/decryption.
///
/// Passwords are returned as [`SensitiveString`] to ensure they are
/// memory-locked (preventing swap to disk) and zeroed on drop.
pub trait PasswordProvider {
    fn password_for_existing_filesystem(&self) -> Result<SensitiveString>;
    fn password_for_new_filesystem(&self) -> Result<SensitiveString>;
}

#[cfg(feature = "testutils")]
pub struct FixedPasswordProvider {
    password: SensitiveString,
}
#[cfg(feature = "testutils")]
impl FixedPasswordProvider {
    pub fn new(password: String) -> Self {
        Self {
            password: SensitiveString::new(password),
        }
    }
}
#[cfg(feature = "testutils")]
impl PasswordProvider for FixedPasswordProvider {
    fn password_for_existing_filesystem(&self) -> Result<SensitiveString> {
        Ok(self.password.clone())
    }

    fn password_for_new_filesystem(&self) -> Result<SensitiveString> {
        Ok(self.password.clone())
    }
}
