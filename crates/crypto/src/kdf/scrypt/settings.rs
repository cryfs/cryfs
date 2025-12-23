//! Scrypt cost settings and presets.

/// Configuration settings for scrypt key derivation.
///
/// These settings control the computational cost of the scrypt algorithm.
/// Higher values provide more security against brute-force attacks but
/// require more memory and CPU time.
///
/// # Memory Usage
///
/// The approximate memory usage can be calculated as:
/// - Single-threaded: `128 * r * (p + n + 1)` bytes
/// - Multi-threaded: `128 * r * p * (n + 2)` bytes
///
/// Where `n = 2^log_n`.
///
/// # Presets
///
/// Several preset configurations are provided:
/// - [`ScryptSettings::PARANOID`]: Maximum security, high resource usage
/// - [`ScryptSettings::DEFAULT`]: Recommended for most use cases
/// - [`ScryptSettings::LOW_MEMORY`]: For memory-constrained environments
/// - [`ScryptSettings::TEST`]: Fast settings for testing only
///
/// # Fields
///
/// - `log_n`: CPU/memory cost as log2(N). N = 2^log_n.
/// - `r`: Block size parameter. Larger values use more memory.
/// - `p`: Parallelization parameter. Higher values allow more parallelism.
/// - `salt_len`: Length of the random salt in bytes.
#[derive(Debug, Clone, Copy)]
pub struct ScryptSettings {
    /// CPU/memory cost parameter as log2(N). Must be < 64.
    pub log_n: u8,
    /// Block size parameter. Affects memory and CPU usage.
    pub r: u32,
    /// Parallelization parameter.
    pub p: u32,
    /// Length of the random salt in bytes.
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
