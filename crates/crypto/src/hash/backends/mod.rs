mod openssl;
pub use openssl::OpensslSha512;

mod sha2;
pub use sha2::Sha2Sha512;

mod libsodium;
pub use libsodium::LibsodiumSha512;
