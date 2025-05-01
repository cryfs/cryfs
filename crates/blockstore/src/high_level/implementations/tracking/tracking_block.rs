use async_trait::async_trait;
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

use crate::{Block, BlockId};
use cryfs_utils::data::Data;

use super::tracking_blockstore::ActionCounts;

/// A wrapper for blocks from an underlying block store
pub struct TrackingBlock<B: Block> {
    underlying_block: B,

    counts: Arc<Mutex<ActionCounts>>,
}

impl<B: Block> TrackingBlock<B> {
    pub fn new(underlying_block: B, counts: Arc<Mutex<ActionCounts>>) -> Self {
        Self {
            underlying_block,
            counts,
        }
    }

    pub(super) fn inner_mut(&mut self) -> &mut B {
        &mut self.underlying_block
    }

    pub(super) fn into_inner(self) -> B {
        self.underlying_block
    }
}

#[async_trait]
impl<B: Block + Send + Sync> Block for TrackingBlock<B> {
    fn block_id(&self) -> &BlockId {
        self.underlying_block.block_id()
    }

    fn data(&self) -> &Data {
        self.counts.lock().unwrap().read += 1;
        self.underlying_block.data()
    }

    fn data_mut(&mut self) -> &mut Data {
        self.counts.lock().unwrap().written += 1;
        self.underlying_block.data_mut()
    }

    async fn resize(&mut self, new_size: usize) {
        self.counts.lock().unwrap().resized += 1;
        self.underlying_block.resize(new_size).await
    }
}

impl<B: Block + Debug> Debug for TrackingBlock<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrackingBlock")
            .field("underlying_block", &self.underlying_block)
            .finish()
    }
}
