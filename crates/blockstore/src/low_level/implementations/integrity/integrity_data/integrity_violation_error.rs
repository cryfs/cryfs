use std::collections::hash_set::HashSet;
use thiserror::Error;

use super::{BlockVersion, ClientId, MaybeClientId};
use crate::BlockId;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum IntegrityViolationError {
    #[error(
        "Integrity Violation: Tried to roll back {block:?} from {from_client:?} (last seen {from_client_last_seen_version:?}) to {to_client:?} (last seen {to_client_last_seen_version:?}) with {actual_version:?}."
    )]
    RollBack {
        block: BlockId,
        from_client: MaybeClientId,
        to_client: ClientId,
        from_client_last_seen_version: Option<BlockVersion>,
        to_client_last_seen_version: BlockVersion,
        actual_version: BlockVersion,
    },

    #[error(
        "Integrity Violation: {id_from_header:?} is stored as {id_from_filename:?}. Did an attacker try to rename some blocks?"
    )]
    WrongBlockId {
        id_from_filename: BlockId,
        id_from_header: BlockId,
    },

    #[error(
        "Integrity Violation: {block:?} should exist but we didn't find it. Did an attacker delete it?"
    )]
    MissingBlock { block: BlockId },

    #[error(
        "Integrity Violation: {blocks:?} should exist but we didn't find them. Did an attacker delete them?"
    )]
    MissingBlocks { blocks: HashSet<BlockId> },
}
