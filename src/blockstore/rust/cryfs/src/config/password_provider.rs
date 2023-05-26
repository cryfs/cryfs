pub trait PasswordProvider {
    // TODO Protect password similar to how we protect EncryptionKey
    fn password_for_existing_filesystem(&self) -> String;
    fn password_for_new_filesystem(&self) -> String;
}

#[cfg(test)]
pub struct FixedPasswordProvider {
    password: String,
}
#[cfg(test)]
impl PasswordProvider for FixedPasswordProvider {
    fn password_for_existing_filesystem(&self) -> String {
        self.password.clone()
    }

    fn password_for_new_filesystem(&self) -> String {
        self.password.clone()
    }
}
