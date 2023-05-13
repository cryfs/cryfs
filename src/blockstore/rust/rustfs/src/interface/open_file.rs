use async_trait::async_trait;

use super::error::FsResult;
use super::node::NodeAttrs;
use crate::utils::{Gid, Mode, NodeKind, Uid};
use std::path::Path;

#[async_trait]
pub trait OpenFile {}
