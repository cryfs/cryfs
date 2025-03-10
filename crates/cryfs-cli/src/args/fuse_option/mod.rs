mod atime_option;
mod permission_option;
use itertools::{Either, Itertools};

use clap::ValueEnum;
use permission_option::FusePermissionOption;

pub use atime_option::AtimeOption;

#[derive(Debug, Clone, Copy)]
pub enum FuseOption {
    AtimeOption(AtimeOption),
    PermissionOption(FusePermissionOption),
}

impl ValueEnum for FuseOption {
    fn value_variants<'a>() -> &'a [Self] {
        let variants = &[
            Self::AtimeOption(AtimeOption::Atime),
            Self::AtimeOption(AtimeOption::Strictatime),
            Self::AtimeOption(AtimeOption::Noatime),
            Self::AtimeOption(AtimeOption::Relatime),
            Self::AtimeOption(AtimeOption::Nodiratime),
            Self::PermissionOption(FusePermissionOption::AllowOther),
            Self::PermissionOption(FusePermissionOption::AllowRoot),
        ];
        assert_eq!(
            variants.len(),
            AtimeOption::value_variants().len() + FusePermissionOption::value_variants().len(),
        );
        variants
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Self::AtimeOption(atime_option) => atime_option.to_possible_value(),
            Self::PermissionOption(permission_option) => permission_option.to_possible_value(),
        }
    }
}

impl FuseOption {
    pub fn partition(this: &[Self]) -> (Vec<AtimeOption>, Vec<FusePermissionOption>) {
        this.iter().partition_map(|option| match option {
            Self::AtimeOption(atime_option) => Either::Left(atime_option),
            Self::PermissionOption(permission_option) => Either::Right(permission_option),
        })
    }
}
