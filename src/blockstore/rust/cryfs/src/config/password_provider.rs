use anyhow::Result;

pub trait PasswordProvider {
    // TODO Protect password similar to how we protect EncryptionKey
    fn password_for_existing_filesystem(&self) -> Result<String>;
    fn password_for_new_filesystem(&self) -> Result<String>;
}

#[cfg(test)]
pub struct FixedPasswordProvider {
    password: String,
}
#[cfg(test)]
impl PasswordProvider for FixedPasswordProvider {
    fn password_for_existing_filesystem(&self) -> Result<String> {
        Ok(self.password.clone())
    }

    fn password_for_new_filesystem(&self) -> Result<String> {
        Ok(self.password.clone())
    }
}
