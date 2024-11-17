use anyhow::Result;
use byte_unit::Byte;
use dialoguer::{Confirm, Select};
use std::path::Path;

use cryfs_filesystem::config::Console;
use cryfs_utils::crypto::kdf::scrypt::ScryptSettings;
use cryfs_version::{Version, VersionInfo};

// TODO Check if dialoguer colorful theme looks better

// TODO Put default block size & cipher into a central place so we can share it with the code that creates file systems with "use default settings? yes"

pub struct InteractiveConsole;

impl Console for InteractiveConsole {
    // TODO Test how all of these look like on the console

    fn ask_migrate_filesystem(
        &self,
        // TODO Pass in a version struct instead of strings
        current_filesystem_format_version: &Version,
        new_filesystem_format_version: &Version,
        cryfs_version: &VersionInfo,
    ) -> Result<bool> {
        let prompt = format!("This filesystem uses file system format {current_filesystem_format_version}. You're running a CryFS version using format {new_filesystem_format_version}. It is recommended to create a new filesystem with CryFS {cryfs_version} and copy your files into it. If you don't want to do that, we can also attempt to migrate the existing filesystem, but that can take a long time, you might not get some of the performance advantages of the new release series, and if the migration fails, your data may be lost. If you decide to continue, please make sure you have a backup of your data.\nDo you want to attempt a migration now?");
        ask_yes_no(&prompt, false)
    }

    fn ask_allow_replaced_filesystem(&self) -> Result<bool> {
        let prompt = "The encryption key differs from the last time we loaded this filesystem. An attacker may have replaced the file system with a different file system.\nDo you want to continue loading?";
        ask_yes_no(prompt, false)
    }

    fn ask_disable_single_client_mode(&self) -> Result<bool> {
        let prompt = "This filesystem is setup to treat missing blocks as integrity violations and therefore only works in single-client mode. You are trying to access it from a different client.\nDo you want to disable this integrity feature and stop treating missing blocks as integrity violations?\nChoosing yes will not affect the confidentiality of your data, but in future you might not notice if an attacker deletes one of your files.";
        ask_yes_no(&prompt, false)
    }

    fn ask_single_client_mode_for_new_filesystem(&self) -> Result<bool> {
        let prompt = "Most integrity checks are enabled by default. However, by default CryFS does not treat missing blocks as integrity violations.\nThat is, if CryFS finds a block missing, it will assume that this is due to a synchronization delay and not because an attacker deleted the block.\nIf you are in a single-client setting, you can let it treat missing blocks as integrity violations, which will ensure that you notice if an attacker deletes one of your files.\nHowever, in this case, you will not be able to use the file system with other devices anymore.\nDo you want to treat missing blocks as integrity violations?";
        ask_yes_no(&prompt, false)
    }

    /// We're in the process of creating a new file system and need to ask the user for the scrypt settings to use
    fn ask_scrypt_settings_for_new_filesystem(&self) -> Result<ScryptSettings> {
        // TODO Allow custom parameters

        fn option(name: &str, opt: ScryptSettings) -> (String, ScryptSettings) {
            (
                format!(
                    "{name} (log(N)={log_n}, r={r}, p={p})",
                    log_n = ScryptSettings::DEFAULT.log_n,
                    r = ScryptSettings::DEFAULT.r,
                    p = ScryptSettings::DEFAULT.p,
                ),
                opt,
            )
        }

        ask_multiple_choice(
            "Please select the scrypt settings to use",
            [
                option("1. Default", ScryptSettings::DEFAULT),
                option("2. Paranoid & Slow", ScryptSettings::PARANOID),
            ]
            .into_iter(),
            0,
        )
    }

    fn ask_cipher_for_new_filesystem(&self) -> Result<String> {
        ask_multiple_choice(
            "Which block cipher do you want to use?",
            cryfs_filesystem::ALL_CIPHERS
                .iter()
                .map(|cipher| (cipher.to_string(), cipher.to_string())),
            0, // TODO Define default cipher somewhere in a constant not by index but by cipher name or enum, and show it correctly in the `--help` as well.
        )
    }

    fn ask_blocksize_bytes_for_new_filesystem(&self) -> Result<Byte> {
        // TODO Allow custom block sizes
        const OPTIONS: &[Byte] = &[kb(4), kb(8), kb(16), kb(32), kb(64), kb(512), mb(1), mb(4)];

        ask_multiple_choice(
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
            2,
        )
    }

    fn ask_create_basedir(&self, path: &Path) -> Result<bool> {
        // TODO Formatting. e.g. can we somehow highlight the path? By color or font? Also check other questions here for what we can do. The console or dialoguer crates could be useful.
        let prompt = format!(
            "Could not find the vault at '{}'. Do you want to create a new vault?",
            path.display()
        );
        ask_yes_no(&prompt, false)
    }

    fn ask_create_mountdir(&self, path: &Path) -> Result<bool> {
        let prompt = format!(
            "Could not find the mount directory at '{}'. Do you want to create a new directory?",
            path.display()
        );
        ask_yes_no(&prompt, false)
    }
}

fn ask_yes_no(prompt: &str, default: bool) -> Result<bool> {
    loop {
        let response = Confirm::new()
            .with_prompt(prompt)
            .default(false)
            .show_default(true)
            .interact_opt()?;
        if let Some(response) = response {
            return Ok(response);
        } else {
            // TODO Use dialoguer for this output
            // TODO Is it actually 'yes' / 'no' or 'y' / 'n'?
            println!("Please enter yes or no");
            continue;
        }
    }
}

fn ask_multiple_choice<S, T>(
    prompt: &str,
    options: impl Iterator<Item = (S, T)>,
    default: usize,
) -> Result<T>
where
    S: ToString,
{
    let (options_str, options_t): (Vec<_>, Vec<_>) = options.unzip();

    let response = Select::new()
        .clear(false) // TODO Should we clear(true)?
        .with_prompt(prompt)
        .default(default)
        .items(&options_str)
        .interact()?;
    // TODO Check we don't have an off-by-one here
    Ok(options_t
        .into_iter()
        .skip(response)
        .next()
        .expect("Out of bounds"))
}

const fn kb(kb: u64) -> Byte {
    Byte::from_u64(kb * 1024)
}

const fn mb(mb: u64) -> Byte {
    Byte::from_u64(mb * 1024 * 1024)
}
