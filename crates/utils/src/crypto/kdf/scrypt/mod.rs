// TODO The scrypt library we're using is not parallelized. Can we find a parallelized version?

mod params;
pub use params::ScryptParams;

mod settings;
pub use settings::ScryptSettings;

pub mod backends;
pub type Scrypt = backends::scrypt::ScryptScrypt;

#[cfg(test)]
mod tests;
