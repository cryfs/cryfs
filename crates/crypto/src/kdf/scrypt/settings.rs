/// Scrypt memory usage (from reading code at <https://github.com/RustCrypto/password-hashes/blob/master/scrypt/src/lib.rs>):
/// * single threaded
///   128 * r * (p + n + 1)
/// * multi threaded
///   128 * r * p * (n + 2)
#[derive(Debug, Clone, Copy)]
pub struct ScryptSettings {
    pub log_n: u8,
    pub r: u32,
    pub p: u32,
    pub salt_len: usize,
}

impl ScryptSettings {
    /// Memory usage: 32GB multi-threaded, 16GB single-threaded (see formula in comment above)
    pub const PARANOID: Self = Self {
        log_n: 24,
        r: 8,
        p: 2,
        salt_len: 32,
    };

    /// Memory usage: 8GB multi-threaded, 1GB single-threaded (see formula in comment above)
    pub const DEFAULT: Self = Self {
        log_n: 20,
        r: 8,
        p: 8,
        salt_len: 32,
    };

    /// Memory usage: 2GB multi-threaded, 500MB single threaded (see formula in comment above)
    pub const LOW_MEMORY: Self = Self {
        log_n: 20,
        r: 4,
        p: 4,
        salt_len: 32,
    };

    /// Memory usage: 256kB multi-threaded, 128kB single-threaded (see formula in comment above)
    // TODO#[cfg(test)]
    pub const TEST: Self = Self {
        log_n: 10,
        r: 1,
        // Use p != r so we find serialization errors
        p: 2,
        salt_len: 32,
    };
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::ScryptSettings;
    use crate::kdf::scrypt::ScryptParams;

    #[rstest]
    fn params_are_valid(
        #[values(
            ScryptSettings::PARANOID,
            ScryptSettings::DEFAULT,
            ScryptSettings::LOW_MEMORY,
            ScryptSettings::TEST
        )]
        settings: ScryptSettings,
    ) {
        let params = ScryptParams::generate(&settings).unwrap();
        scrypt::Params::new(params.log_n(), params.r(), params.p())
            .expect("Invalid scrypt parameters");
    }
}
