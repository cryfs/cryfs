use std::fmt::{self, Debug, Display, Formatter};

// TODO Complete this into what the C++ gitversion module was doing

#[derive(Clone, Copy)]
pub struct Version<'a> {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: &'a str,
}

impl Debug for Version<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}{}",
            self.major, self.minor, self.patch, self.prerelease
        )
    }
}

impl Display for Version<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}{}",
            self.major, self.minor, self.patch, self.prerelease
        )
    }
}

#[macro_export]
macro_rules! get_package_version {
    () => {
        Version {
            major: env!("CARGO_PKG_VERSION_MAJOR")
                .parse()
                .expect("CARGO_PKG_VERSION_MAJOR is not a number"),
            minor: env!("CARGO_PKG_VERSION_MINOR")
                .parse()
                .expect("CARGO_PKG_VERSION_MINOR is not a number"),
            patch: env!("CARGO_PKG_VERSION_PATCH")
                .parse()
                .expect("CARGO_PKG_VERSION_PATCH is not a number"),
            prerelease: env!("CARGO_PKG_VERSION_PRE"),
        }
    };
}
