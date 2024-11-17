use byte_unit::{Byte, UnitType};
use std::fmt::Display;

use cryfs_filesystem::config::ConfigLoadResult;

// TODO Integration test the outputs of print_config
pub fn print_config(config: &ConfigLoadResult) {
    fn print_value<T: Display + Eq>(old_value: T, new_value: T) {
        if old_value == new_value {
            print!("{}", old_value);
        } else {
            print!("{} -> {}", old_value, new_value);
        }
    }

    fn format_bytes(bytes: Byte) -> String {
        format!(
            "{} ({} bytes)",
            bytes.get_appropriate_unit(UnitType::Binary),
            bytes.as_u64(),
        )
    }

    println!("----------------------------------------------------");
    println!("Filesystem configuration:");
    println!("----------------------------------------------------");
    print!("- Filesystem format version: ");
    print_value(
        &config.old_config.format_version,
        &config.config.config().format_version,
    );
    print!("\n- Created with: CryFS ");
    print_value(
        &config.old_config.created_with_version,
        &config.config.config().created_with_version,
    );
    print!("\n- Last opened with: CryFS ");
    print_value(
        &config.old_config.last_opened_with_version,
        &config.config.config().last_opened_with_version,
    );
    print!("\n- Cipher: ");
    print_value(&config.old_config.cipher, &config.config.config().cipher);
    print!("\n- Blocksize: ");
    print_value(
        format_bytes(config.old_config.blocksize),
        format_bytes(config.config.config().blocksize),
    );
    print!("\n- Filesystem Id: ");
    print_value(
        config.old_config.filesystem_id.to_hex(),
        config.config.config().filesystem_id.to_hex(),
    );

    println!("\n----------------------------------------------------");
}
