use derive_more::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, From, Into,
    Not, Sub, SubAssign, Sum,
};
use std::ops::{Div, DivAssign, Mul, MulAssign};

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

#[allow(non_snake_case)]
const fn S_ISREG(mode: Mode) -> bool {
    return (mode.0 & S_IFMT.0) == S_IFREG.0;
}

#[allow(non_snake_case)]
const fn S_ISDIR(mode: Mode) -> bool {
    return (mode.0 & S_IFMT.0) == S_IFDIR.0;
}

#[allow(non_snake_case)]
const fn S_ISLNK(mode: Mode) -> bool {
    return (mode.0 & S_IFMT.0) == S_IFLNK.0;
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    From,
    Into,
    BitAnd,
    BitAndAssign,
    BitOr,
    BitOrAssign,
    BitXor,
    BitXorAssign,
    Not,
)]
pub struct Mode(u32);

impl Mode {
    pub const fn add_file_flag(mut self) -> Self {
        self.0 |= S_IFREG.0;
        self
    }

    pub const fn add_dir_flag(mut self) -> Self {
        self.0 |= S_IFDIR.0;
        self
    }

    pub const fn add_symlink_flag(mut self) -> Self {
        self.0 |= S_IFLNK.0;
        self
    }

    pub const fn add_user_read_flag(mut self) -> Self {
        self.0 |= S_IRUSR.0;
        self
    }

    pub const fn add_user_write_flag(mut self) -> Self {
        self.0 |= S_IWUSR.0;
        self
    }

    pub const fn add_user_exec_flag(mut self) -> Self {
        self.0 |= S_IXUSR.0;
        self
    }

    pub const fn add_group_read_flag(mut self) -> Self {
        self.0 |= S_IRGRP.0;
        self
    }

    pub const fn add_group_write_flag(mut self) -> Self {
        self.0 |= S_IWGRP.0;
        self
    }

    pub const fn add_group_exec_flag(mut self) -> Self {
        self.0 |= S_IXGRP.0;
        self
    }

    pub const fn add_other_read_flag(mut self) -> Self {
        self.0 |= S_IROTH.0;
        self
    }

    pub const fn add_other_write_flag(mut self) -> Self {
        self.0 |= S_IWOTH.0;
        self
    }

    pub const fn add_other_exec_flag(mut self) -> Self {
        self.0 |= S_IXOTH.0;
        self
    }

    pub const fn node_kind(self) -> NodeKind {
        if S_ISREG(self) {
            NodeKind::File
        } else if S_ISDIR(self) {
            NodeKind::Dir
        } else if S_ISLNK(self) {
            NodeKind::Symlink
        } else {
            // TODO What to do here? Maybe we should instead check this invariant when Mode objects get created or modified.
            panic!("invalid mode")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into)]
pub struct Uid(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into)]
pub struct Gid(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into, Add, AddAssign, Sub, SubAssign, Sum)]
pub struct NumBytes(u64);

impl Mul<u64> for NumBytes {
    type Output = NumBytes;
    fn mul(self, rhs: u64) -> Self::Output {
        NumBytes(self.0 * rhs)
    }
}
impl MulAssign<u64> for NumBytes {
    fn mul_assign(&mut self, rhs: u64) {
        self.0 *= rhs;
    }
}
impl Mul<NumBytes> for u64 {
    type Output = NumBytes;
    fn mul(self, rhs: NumBytes) -> Self::Output {
        NumBytes(self * rhs.0)
    }
}
impl Div<u64> for NumBytes {
    type Output = NumBytes;
    fn div(self, rhs: u64) -> Self::Output {
        NumBytes(self.0 / rhs)
    }
}
impl DivAssign<u64> for NumBytes {
    fn div_assign(&mut self, rhs: u64) {
        self.0 /= rhs;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    File,
    Dir,
    Symlink,
}
