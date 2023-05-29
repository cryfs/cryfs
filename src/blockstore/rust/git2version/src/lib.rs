pub const COMMIT_ID_SHORT_HASH_LENGTH: usize = 10;

#[cfg(feature = "build")]
mod git_helpers;
#[cfg(feature = "build")]
mod gitinfo_owned;
#[cfg(feature = "build")]
pub use gitinfo_owned::{get_git_info, GitInfoOwned};

mod gitinfo;
pub use gitinfo::GitInfo;

mod proxy;

// We need to re-export this because our macros use it
#[cfg(feature = "build")]
pub use git2;
pub use konst;
