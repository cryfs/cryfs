// TODO The scrypt library we're using is not parallelized. Can we find a parallelized version?

mod params;
pub use params::ScryptParams;

mod settings;
pub use settings::ScryptSettings;

mod backends;
pub use backends::scrypt::Scrypt;

#[cfg(test)]
mod tests;
