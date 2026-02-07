use anyhow::{Result, ensure};
use console::style;

use cryfs_config::config::PasswordProvider;
use cryfs_crypto::sensitive_string::SensitiveString;

pub struct InteractivePasswordProvider;

impl PasswordProvider for InteractivePasswordProvider {
    fn password_for_existing_filesystem(&self) -> Result<SensitiveString> {
        // TODO Check how this flow looks like when actually running
        loop {
            println!();
            let password = ask_password_from_console("Password: ")?;
            match check_password(&password) {
                Ok(()) => {
                    return Ok(password);
                }
                Err(err) => {
                    println!("Error: {}", err);
                    continue;
                }
            }
        }
    }

    fn password_for_new_filesystem(&self) -> Result<SensitiveString> {
        // TODO Check how this flow looks like when actually running
        loop {
            println!();
            let password = ask_password_from_console("Password: ")?;
            match check_password(&password) {
                Ok(()) => {
                    let confirm_password = ask_password_from_console("Confirm Password: ")?;
                    if *password != *confirm_password {
                        // TODO Error message formatting (e.g. colorization), here and above
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
    fn password_for_existing_filesystem(&self) -> Result<SensitiveString> {
        let password = ask_password_from_console("Password: ")?;
        check_password(&password)?;
        Ok(password)
    }

    fn password_for_new_filesystem(&self) -> Result<SensitiveString> {
        let password = ask_password_from_console("Password: ")?;
        check_password(&password)?;
        Ok(password)
    }
}

fn ask_password_from_console(prompt: &str) -> Result<SensitiveString> {
    let indent = "  ";
    let prompt = format!("{indent}{prompt}");
    let password = rpassword::prompt_password(style(prompt).bold())?;
    // Wrap in SensitiveString immediately to get mlock + zeroize-on-drop protection
    Ok(SensitiveString::new(password))
}

fn check_password(password: &str) -> Result<()> {
    ensure!(
        !password.is_empty(),
        "Invalid password. Password cannot be empty."
    );
    Ok(())
}

// TODO Tests
