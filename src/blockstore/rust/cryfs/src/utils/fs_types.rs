use binrw::{BinRead, BinWrite};
use derive_more::{BitAnd, BitAndAssign, BitOr, BitOrAssign, From, Into};

// TODO This should probably live in fspp not in cryfs

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
    fn S_ISREG(self) -> bool {
        (self & S_IFMT) == S_IFREG
    }

    #[allow(non_snake_case)]
    fn S_ISDIR(self) -> bool {
        (self & S_IFMT) == S_IFDIR
    }

    #[allow(non_snake_case)]
    fn S_ISLNK(self) -> bool {
        (self & S_IFMT) == S_IFLNK
    }

    pub fn add_file_flag(&mut self) -> &mut Self {
        *self |= S_IFREG;
        self
    }

    pub fn add_dir_flag(&mut self) -> &mut Self {
        *self |= S_IFDIR;
        self
    }

    pub fn add_symlink_flag(&mut self) -> &mut Self {
        *self |= S_IFLNK;
        self
    }

    pub fn add_user_read_flag(&mut self) -> &mut Self {
        *self |= S_IRUSR;
        self
    }

    pub fn add_user_write_flag(&mut self) -> &mut Self {
        *self |= S_IWUSR;
        self
    }

    pub fn add_user_exec_flag(&mut self) -> &mut Self {
        *self |= S_IXUSR;
        self
    }

    pub fn add_group_read_flag(&mut self) -> &mut Self {
        *self |= S_IRGRP;
        self
    }

    pub fn add_group_write_flag(&mut self) -> &mut Self {
        *self |= S_IWGRP;
        self
    }

    pub fn add_group_exec_flag(&mut self) -> &mut Self {
        *self |= S_IXGRP;
        self
    }

    pub fn add_other_read_flag(&mut self) -> &mut Self {
        *self |= S_IROTH;
        self
    }

    pub fn add_other_write_flag(&mut self) -> &mut Self {
        *self |= S_IWOTH;
        self
    }

    pub fn add_other_exec_flag(&mut self) -> &mut Self {
        *self |= S_IXOTH;
        self
    }

    pub fn has_file_flag(self) -> bool {
        self.S_ISREG()
    }

    pub fn has_dir_flag(self) -> bool {
        self.S_ISDIR()
    }

    pub fn has_symlink_flag(self) -> bool {
        self.S_ISLNK()
    }
}
