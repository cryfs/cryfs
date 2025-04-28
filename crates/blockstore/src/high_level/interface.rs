use anyhow::Result;

use crate::BlockId;
use cryfs_utils::data::Data;

pub trait Block {
    type BlockStore;

    fn block_id(&self) -> &BlockId;
    fn data(&self) -> &Data;
    fn data_mut(&mut self) -> &mut Data;
    async fn resize(&mut self, new_size: usize);
    // TODO This is a weird API. We should probably change this so that code calls block_store.remove(block) instead. Then we can remove the `type BlockStore` above.
    async fn remove(self, block_store: &Self::BlockStore) -> Result<()>;
}
