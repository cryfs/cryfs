#[derive(Debug)]
pub struct ScryptSettings {
    pub log_n: u8,
    pub r: u32,
    pub p: u32,
    pub salt_len: usize,
}

impl ScryptSettings {
    pub const PARANOID: Self = Self {
        log_n: 20,
        r: 8,
        p: 16,
        salt_len: 32,
    };

    pub const DEFAULT: Self = Self {
        log_n: 20,
        r: 4,
        p: 8,
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
