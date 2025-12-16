use byte_unit::{Byte, UnitType};
use console::{StyledObject, style};
use std::fmt::Display;

use cryfs_config::config::ConfigLoadResult;

// TODO Integration test the outputs of print_config
pub fn print_config(config: &ConfigLoadResult) {
    fn print_value<T: Display + Eq>(old_value: T, new_value: T) {
        if old_value == new_value {
            print!("{}", format_value(&old_value.to_string()));
        } else {
            print!(
                "{} -> {}",
                format_value(&old_value.to_string()),
                format_value(&new_value.to_string())
            );
        }
    }

    fn format_bytes(bytes: Byte) -> String {
        format!(
            "{} ({} bytes)",
            bytes.get_appropriate_unit(UnitType::Binary),
            bytes.as_u64(),
        )
    }

    fn format_key(name: &str) -> StyledObject<&str> {
        style(name).bold().blue()
    }

    fn format_value(name: &str) -> StyledObject<&str> {
        style(name).yellow()
    }

    println!("\n  {}", style("Filesystem configuration:").bold());
    println!("  ----------------------------------------------------");
    print!("  • {} ", format_key("Filesystem format version:"));
    print_value(
        &config.old_config.format_version,
        &config.config.config().format_version,
    );
    print!(
        "\n  • {} {} ",
        format_key("Created with:"),
        format_value("CryFS"),
    );
    print_value(
        &config.old_config.created_with_version,
        &config.config.config().created_with_version,
    );
    print!(
        "\n  • {} {} ",
        format_key("Last opened with:"),
        format_value("CryFS"),
    );
    print_value(
        &config.old_config.last_opened_with_version,
        &config.config.config().last_opened_with_version,
    );
    print!("\n  • {} ", format_key("Cipher:"));
    print_value(&config.old_config.cipher, &config.config.config().cipher);
    print!("\n  • {} ", format_key("Blocksize:"));
    print_value(
        format_bytes(config.old_config.blocksize),
        format_bytes(config.config.config().blocksize),
    );
    print!("\n  • {} ", format_key("Filesystem Id:"));
    print_value(
        config.old_config.filesystem_id.to_hex(),
        config.config.config().filesystem_id.to_hex(),
    );

    println!("\n  ----------------------------------------------------\n");
}
