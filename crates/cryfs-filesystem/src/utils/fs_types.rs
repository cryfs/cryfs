use binrw::{BinRead, BinWrite};
use derive_more::{BitAnd, BitAndAssign, BitOr, BitOrAssign, From, Into};

// TODO Unify this with cryfs_rustfs types

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, BinRead, BinWrite, From, Into)]
pub struct Uid(u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, BinRead, BinWrite, From, Into)]
pub struct Gid(u32);

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Debug,
    BitAndAssign,
    BitAnd,
    BitOrAssign,
    BitOr,
    BinRead,
    BinWrite,
    From,
    Into,
)]
pub struct Mode(u32);

const S_IFMT: Mode = Mode(0o170000);
const S_IFDIR: Mode = Mode(0o040000);
const S_IFREG: Mode = Mode(0o100000);
const S_IFLNK: Mode = Mode(0o120000);

const S_IRUSR: Mode = Mode(0o000400);
const S_IWUSR: Mode = Mode(0o000200);
const S_IXUSR: Mode = Mode(0o000100);
const S_IRGRP: Mode = Mode(0o000040);
const S_IWGRP: Mode = Mode(0o000020);
const S_IXGRP: Mode = Mode(0o000010);
const S_IROTH: Mode = Mode(0o000004);
const S_IWOTH: Mode = Mode(0o000002);
const S_IXOTH: Mode = Mode(0o000001);

impl Mode {
    // TODO Make functions const
    pub const fn zero() -> Mode {
        Mode(0)
    }

    #[allow(non_snake_case)]
    const fn S_ISREG(self) -> bool {
        (self.0 & S_IFMT.0) == S_IFREG.0
    }

    #[allow(non_snake_case)]
    const fn S_ISDIR(self) -> bool {
        (self.0 & S_IFMT.0) == S_IFDIR.0
    }

    #[allow(non_snake_case)]
    const fn S_ISLNK(self) -> bool {
        (self.0 & S_IFMT.0) == S_IFLNK.0
    }

    pub const fn with_file_flag(mut self) -> Self {
        self.0 |= S_IFREG.0;
        self
    }

    pub const fn with_dir_flag(mut self) -> Self {
        self.0 |= S_IFDIR.0;
        self
    }

    pub const fn with_symlink_flag(mut self) -> Self {
        self.0 |= S_IFLNK.0;
        self
    }

    pub const fn with_user_read_flag(mut self) -> Self {
        self.0 |= S_IRUSR.0;
        self
    }

    pub const fn with_user_write_flag(mut self) -> Self {
        self.0 |= S_IWUSR.0;
        self
    }

    pub const fn with_user_exec_flag(mut self) -> Self {
        self.0 |= S_IXUSR.0;
        self
    }

    pub const fn with_group_read_flag(mut self) -> Self {
        self.0 |= S_IRGRP.0;
        self
    }

    pub const fn with_group_write_flag(mut self) -> Self {
        self.0 |= S_IWGRP.0;
        self
    }

    pub const fn with_group_exec_flag(mut self) -> Self {
        self.0 |= S_IXGRP.0;
        self
    }

    pub const fn with_other_read_flag(mut self) -> Self {
        self.0 |= S_IROTH.0;
        self
    }

    pub const fn with_other_write_flag(mut self) -> Self {
        self.0 |= S_IWOTH.0;
        self
    }

    pub const fn with_other_exec_flag(mut self) -> Self {
        self.0 |= S_IXOTH.0;
        self
    }

    pub const fn has_file_flag(self) -> bool {
        self.S_ISREG()
    }

    pub const fn has_dir_flag(self) -> bool {
        self.S_ISDIR()
    }

    pub const fn has_symlink_flag(self) -> bool {
        self.S_ISLNK()
    }
}
