use std::collections::hash_set::HashSet;
use std::path::PathBuf;
use thiserror::Error;

use super::{BlockVersion, MaybeClientId, ClientId};
use crate::blockstore::BlockId;

#[derive(Error, Debug)]
pub enum IntegrityViolationError {
    #[error(
        "Integrity Violation: Tried to roll back block {block:?} from client {from_client:?} version {from_version:?} to client {to_client:?} version {to_version:?}."
    )]
    RollBack {
        block: BlockId,
        from_client: MaybeClientId,
        to_client: ClientId,
        from_version: BlockVersion,
        to_version: BlockVersion,
    },

    #[error("Integrity Violation: Block {id_from_header:?} is stored as block {id_from_filename:?}. Did an attacker try to rename some blocks?")]
    WrongBlockId {
        id_from_filename: BlockId,
        id_from_header: BlockId,
    },

    #[error("Integrity Violation: Block {block:?} should exist but we didn't find it. Did an attacker delete it?")]
    MissingBlock { block: BlockId },

    #[error("Integrity Violation: Blocks {blocks:?} should exist but we didn't find them. Did an attacker delete them?")]
    MissingBlocks { blocks: HashSet<BlockId> },

    #[error("There was an integrity violation detected. Preventing any further access to the file system. This can either happen if an attacker changed your files or rolled back the file system to a previous state, but it can also happen if you rolled back the file system yourself, for example restored a backup. If you want to reset the integrity data (i.e. accept changes made by a potential attacker), please delete the following file before re-mounting it: {integrity_file_path}")]
    IntegrityViolationInPreviousRun { integrity_file_path: PathBuf },
}
