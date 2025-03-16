use anyhow::{Result, ensure};
use console::style;

use cryfs_filesystem::config::PasswordProvider;

// TODO Protect password similar to how we protect EncryptionKey (mprotect, zero on drop, ...). The rpassword crate actually has an internal class `SafeString` but then they extract it from that before returning :(

pub struct InteractivePasswordProvider;

impl PasswordProvider for InteractivePasswordProvider {
    fn password_for_existing_filesystem(&self) -> Result<String> {
        // TODO Check how this flow looks like when actually running
        loop {
            let password = ask_password_from_console("Password: ")?;
            match check_password(&password) {
                Ok(()) => return Ok(password),
                Err(err) => {
                    println!("Error: {}", err);
                    continue;
                }
            }
        }
    }

    fn password_for_new_filesystem(&self) -> Result<String> {
        // TODO Check how this flow looks like when actually running
        loop {
            let password = ask_password_from_console("Password: ")?;
            match check_password(&password) {
                Ok(()) => {
                    let confirm_password = ask_password_from_console("Confirm Password: ")?;
                    if password != confirm_password {
                        println!("Passwords do not match. Please try again.");
                        continue;
                    }
                    return Ok(password);
                }
                Err(err) => {
                    println!("Error: {}", err);
                    continue;
                }
            }
        }
    }
}

pub struct NoninteractivePasswordProvider;

impl PasswordProvider for NoninteractivePasswordProvider {
    fn password_for_existing_filesystem(&self) -> Result<String> {
        let password = ask_password_from_console("Password: ")?;
        check_password(&password)?;
        Ok(password)
    }

    fn password_for_new_filesystem(&self) -> Result<String> {
        let password = ask_password_from_console("Password: ")?;
        check_password(&password)?;
        Ok(password)
    }
}

fn ask_password_from_console(prompt: &str) -> Result<String> {
    let indent = "  ";
    let prompt = format!("{indent}{prompt}");
    let password = rpassword::prompt_password(style(prompt).bold())?;
    Ok(password)
}

fn check_password(password: &str) -> Result<()> {
    ensure!(
        !password.is_empty(),
        "Invalid password. Password cannot be empty."
    );
    Ok(())
}

// TODO Tests
