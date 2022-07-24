use anyhow::{anyhow, ensure, Result};

use crate::blobstore::on_blocks::data_node_store::{DataNode, DataNodeStore};
use crate::blockstore::low_level::BlockStore;
use crate::utils::async_drop::{AsyncDropArc, AsyncDropGuard};

use super::size_cache::SizeCache;

pub struct DataTree<B: BlockStore + Send + Sync> {
    // The lock on the root node also ensures that there never are two [DataTree] instances for the same tree
    // &mut self in all the methods makes sure we don't run into race conditions where
    // one task modifies a tree we're currently trying to read somewhere else.
    // TODO Think about whether we can allow some kind of concurrency, e.g. multiple concurrent reads
    // (but we may have to think about how that interacts with the size_cache since even reads might write to that)
    root_node: DataNode<B>,
    node_store: AsyncDropGuard<AsyncDropArc<DataNodeStore<B>>>,
    num_bytes_cache: SizeCache,
}

impl<B: BlockStore + Send + Sync> DataTree<B> {
    pub fn new(
        root_node: DataNode<B>,
        node_store: AsyncDropGuard<AsyncDropArc<DataNodeStore<B>>>,
    ) -> Self {
        Self {
            root_node,
            node_store,
            num_bytes_cache: SizeCache::SizeUnknown,
        }
    }

    pub async fn num_bytes(&mut self) -> Result<u64> {
        self.num_bytes_cache
            .get_or_calculate_num_bytes(&self.node_store, &self.root_node)
            .await
    }

    pub async fn read_bytes(&mut self, offset: u64, target: &mut [u8]) -> Result<()> {
        let num_bytes = self.num_bytes().await?;
        let target_len = u64::try_from(target.len()).unwrap();
        let read_end = offset.checked_add(target_len).ok_or_else(|| {
            anyhow!(
                "Overflow in offset+target.len(): {}+{}",
                offset,
                target.len()
            )
        })?;
        ensure!(read_end <= num_bytes, "DataTree::read_bytes() tried to read range {}..{} but only has {} bytes stored. Use try_read_bytes() if this should be allowed.", offset, read_end, num_bytes);
        self._do_read_bytes(offset, target).await?;
        Ok(())
    }

    pub async fn try_read_bytes(&mut self, offset: u64, target: &mut [u8]) -> Result<usize> {
        //TODO Quite inefficient to call num_bytes() here, because that has to traverse the tree
        let num_bytes = self.num_bytes().await?;
        let real_target_len: usize = target
            .len()
            .min(usize::try_from(num_bytes.saturating_sub(offset)).unwrap_or(usize::MAX));
        let real_target = &mut target[..real_target_len];
        self._do_read_bytes(offset, real_target).await?;
        Ok(real_target_len)
    }

    async fn _do_read_bytes(&mut self, offset: u64, target: &mut [u8]) -> Result<()> {
        todo!()
    }
}
