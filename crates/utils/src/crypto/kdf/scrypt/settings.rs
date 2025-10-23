#[derive(Debug, Clone, Copy)]
pub struct ScryptSettings {
    pub log_n: u8,
    pub r: u32,
    pub p: u32,
    pub salt_len: usize,
}

impl ScryptSettings {
    pub const PARANOID: Self = Self {
        log_n: 24,
        r: 8,
        p: 1,
        salt_len: 32,
    };

    pub const DEFAULT: Self = Self {
        log_n: 22,
        r: 8,
        p: 1,
        salt_len: 32,
    };

    pub const LOW_MEMORY: Self = Self {
        log_n: 22,
        r: 2,
        p: 4,
        salt_len: 32,
    };

    // #[cfg(test)]
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
    use crate::crypto::kdf::scrypt::ScryptParams;

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
