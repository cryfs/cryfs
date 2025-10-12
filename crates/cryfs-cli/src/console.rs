use anyhow::Result;
use byte_unit::Byte;
use dialoguer::{Confirm, Select, console::style, theme::ColorfulTheme};
use once_cell::unsync::OnceCell;
use std::{fmt::Display, path::Path};

use cryfs_filesystem::config::Console;
use cryfs_utils::crypto::kdf::scrypt::ScryptSettings;
use cryfs_version::{Version, VersionInfo};

// TODO Put default block size & cipher into a central place so we can share it with the code that creates file systems with "use default settings? yes"

pub struct InteractiveConsole {
    /// First time we ask for a creation setting, we ask the user if they want to use the default settings.
    /// If they say yes, we store the answer in this variable to avoid asking again.
    /// This only applies to filesystem creation settings, not to other questions we ask the user.
    use_default_creation_settings: OnceCell<bool>,
}

impl InteractiveConsole {
    pub fn new() -> Self {
        Self {
            use_default_creation_settings: OnceCell::new(),
        }
    }

    fn _use_default_creation_settings(&self) -> Result<bool> {
        self.use_default_creation_settings
            .get_or_try_init(|| ask_yes_no(Some("You can either manually configure the file system, or use the default settings."), "Use default settings?", true))
            .copied()
    }
}

impl Console for InteractiveConsole {
    // TODO Test how all of these look like on the console

    fn ask_migrate_filesystem(
        &self,
        current_filesystem_format_version: &Version<&str>,
        new_filesystem_format_version: &Version<&str>,
        cryfs_version: &VersionInfo<&str>,
    ) -> Result<bool> {
        let explanation = format!(
            "This filesystem uses file system format {current_filesystem_format_version}. You're running a CryFS version using format {new_filesystem_format_version}. It is recommended to create a new filesystem with CryFS {cryfs_version} and copy your files into it. If you don't want to do that, we can also attempt to migrate the existing filesystem, but that can take a long time, you might not get some of the performance advantages of the new release series, and if the migration fails, your data may be lost. If you decide to continue, please make sure you have a backup of your data."
        );
        let prompt = "Do you want to attempt a migration now?";
        ask_yes_no(Some(&explanation), &prompt, false)
    }

    fn ask_allow_replaced_filesystem(&self) -> Result<bool> {
        let explanation = "The filesystem id in the config file is different to the last time we loaded a filesystem from this basedir. This can be genuine if you replaced the filesystem with a different one. If you didn't do that, it is possible that an attacker did.";
        let prompt = "Do you want to continue loading the file system?";
        ask_yes_no(Some(explanation), prompt, false)
    }

    fn ask_allow_changed_encryption_key(&self) -> Result<bool> {
        let explanation = "The encryption key differs from the last time we loaded this filesystem. An attacker may have replaced the file system with a different file system.";
        let prompt = "Do you want to continue loading?";
        ask_yes_no(Some(explanation), prompt, false)
    }

    fn ask_disable_single_client_mode(&self) -> Result<bool> {
        let explanation = "This filesystem is setup to treat missing blocks as integrity violations and therefore only works in single-client mode. You are trying to access it from a different client.\nYou can disable this integrity feature and stop treating missing blocks as integrity violations?\nChoosing yes will not affect the confidentiality of your data, but in future you might not notice if an attacker deletes one of your files.";
        let prompt = "Do you want to stop treating missing blocks as integrity violations?";
        ask_yes_no(Some(explanation), &prompt, false)
    }

    fn ask_single_client_mode_for_new_filesystem(&self) -> Result<bool> {
        const DEFAULT: bool = false;

        if self._use_default_creation_settings()? {
            return Ok(DEFAULT);
        }

        let explanation = "Most integrity checks are enabled by default. However, by default CryFS does not treat missing blocks as integrity violations.\nThat is, if CryFS finds a block missing, it will assume that this is due to a synchronization delay and not because an attacker deleted the block.\nIf you are in a single-client setting, you can let it treat missing blocks as integrity violations, which will ensure that you notice if an attacker deletes one of your files.\nHowever, in this case, you will not be able to use the file system with other devices anymore.";
        let prompt = "Do you want to treat missing blocks as integrity violations?";
        ask_yes_no(Some(explanation), &prompt, DEFAULT)
    }

    /// We're in the process of creating a new file system and need to ask the user for the scrypt settings to use
    fn ask_scrypt_settings_for_new_filesystem(&self) -> Result<ScryptSettings> {
        let options = [
            option(
                "1. Low Memory: less secure, but works on devices with less memory",
                ScryptSettings::LOW_MEMORY,
            ),
            option("2. Default", ScryptSettings::DEFAULT),
            option(
                "3. Paranoid: more secure, but mounting will be slow",
                ScryptSettings::PARANOID,
            ),
        ];
        const DEFAULT_INDEX: usize = 1;

        if self._use_default_creation_settings()? {
            return Ok(options[DEFAULT_INDEX].1);
        }

        // TODO Allow custom parameters
        // TODO Add command line parameters for scrypt settings

        fn option(name: &str, opt: ScryptSettings) -> (String, ScryptSettings) {
            (
                format!(
                    "{name} (log(N)={log_n}, r={r}, p={p})",
                    log_n = opt.log_n,
                    r = opt.r,
                    p = opt.p,
                ),
                opt,
            )
        }

        ask_multiple_choice(
            Some(
                "Scrypt is used to derive an encryption key from your password, to protect your file system against brute force attacks",
            ),
            "Please select the scrypt settings to use",
            options.into_iter(),
            DEFAULT_INDEX,
        )
    }

    fn ask_cipher_for_new_filesystem(&self) -> Result<String> {
        // TODO Define default cipher somewhere in a constant not by index but by cipher name or enum, and show it correctly in the `--help` as well. Same for blocksize bytes.
        const DEFAULT_CIPHER_INDEX: usize = 0;

        if self._use_default_creation_settings()? {
            return Ok(cryfs_filesystem::ALL_CIPHERS[DEFAULT_CIPHER_INDEX].to_string());
        }

        ask_multiple_choice(
            None,
            "Which cipher do you want to use to encrypt your file system?",
            cryfs_filesystem::ALL_CIPHERS
                .iter()
                .map(|cipher| (cipher.to_string(), cipher.to_string())),
            DEFAULT_CIPHER_INDEX,
        )
    }

    fn ask_blocksize_bytes_for_new_filesystem(&self) -> Result<Byte> {
        // TODO Allow custom block sizes. Careful to use Byte::parse_str(ignore_case=true) or it will interpret smaller case letters as bits.
        const OPTIONS: &[Byte] = &[kb(4), kb(8), kb(16), kb(32), kb(64), kb(512), mb(1), mb(4)];
        const DEFAULT_BLOCKSIZE_INDEX: usize = 2;

        if self._use_default_creation_settings()? {
            return Ok(OPTIONS[DEFAULT_BLOCKSIZE_INDEX]);
        }

        ask_multiple_choice(
            Some("CryFS splits all data into same-size blocks to hide file and directory sizes."),
            "Which block size do you want to use?",
            OPTIONS.iter().map(|option| {
                (
                    format!(
                        "{}",
                        option.get_appropriate_unit(byte_unit::UnitType::Binary)
                    ),
                    *option,
                )
            }),
            DEFAULT_BLOCKSIZE_INDEX,
        )
    }

    fn ask_create_basedir(&self, path: &Path) -> Result<bool> {
        // TODO Formatting. e.g. can we somehow highlight the path? By color or font? Also check other questions here for what we can do. The console or dialoguer crates could be useful.
        let explanation = format!("Could not find the vault at '{}'.", path.display());
        let prompt = "Do you want to create a new vault?";
        ask_yes_no(Some(&explanation), &prompt, true)
    }

    fn ask_create_mountdir(&self, path: &Path) -> Result<bool> {
        let explanation = format!(
            "Could not find the mount directory at '{}'.",
            path.display()
        );
        let prompt = "Do you want to create a new directory?";
        ask_yes_no(Some(&explanation), &prompt, true)
    }
}

fn ask_yes_no(explanation: Option<&str>, prompt: &str, default: bool) -> Result<bool> {
    println!();
    if let Some(explanation) = explanation {
        println!("{}", format_explanation(explanation));
    }
    loop {
        let response = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("{prompt}"))
            .default(default)
            .show_default(true)
            .interact_opt()?;
        if let Some(response) = response {
            return Ok(response);
        } else {
            // TODO Output formatting
            println!("Please press [y] for yes or [n] for no");
            continue;
        }
    }
}

fn ask_multiple_choice<S, T>(
    explanation: Option<&str>,
    prompt: &str,
    options: impl Iterator<Item = (S, T)>,
    default: usize,
) -> Result<T>
where
    S: ToString + Display,
{
    let (options_str, options_t): (Vec<_>, Vec<_>) = options.unzip();

    println!();
    if let Some(explanation) = explanation {
        println!("{}", format_explanation(explanation));
    }

    let response = Select::with_theme(&ColorfulTheme::default())
        .clear(false) // TODO Should we clear(true)?
        .with_prompt(format!("{prompt}"))
        .default(default)
        .items(&options_str)
        .interact()?;
    let response = options_t
        .into_iter()
        .skip(response)
        .next()
        .expect("Out of bounds");
    Ok(response)
}

fn format_explanation(explanation: &str) -> String {
    let indent = "  ";
    let explanation = explanation.replace("\n", &format!("\n{indent}"));
    format!("{indent}{}", style(explanation).dim())
}

const fn kb(kb: u64) -> Byte {
    Byte::from_u64(kb * 1024)
}

const fn mb(mb: u64) -> Byte {
    Byte::from_u64(mb * 1024 * 1024)
}
