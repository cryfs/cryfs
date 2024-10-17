use anyhow::Result;

pub trait PasswordProvider {
    // TODO Protect password similar to how we protect EncryptionKey
    fn password_for_existing_filesystem(&self) -> Result<String>;
    fn password_for_new_filesystem(&self) -> Result<String>;
}

#[cfg(any(test, feature = "testutils"))]
pub struct FixedPasswordProvider {
    password: String,
}
#[cfg(any(test, feature = "testutils"))]
impl FixedPasswordProvider {
    pub fn new(password: String) -> Self {
        Self { password }
    }
}
#[cfg(any(test, feature = "testutils"))]
impl PasswordProvider for FixedPasswordProvider {
    fn password_for_existing_filesystem(&self) -> Result<String> {
        Ok(self.password.clone())
    }

    fn password_for_new_filesystem(&self) -> Result<String> {
        Ok(self.password.clone())
    }
}
