mod data_node_store;
mod data_tree_store;

mod blob_on_blocks;
mod blobstore_on_blocks;

pub use blob_on_blocks::BlobOnBlocks;
pub use blobstore_on_blocks::BlobStoreOnBlocks;
pub use data_node_store::{DataInnerNode, DataLeafNode, DataNode, DataNodeStore};
pub use data_tree_store::{DataTree, DataTreeStore, LoadNodeError};

#[cfg(test)]
mod test_as_blockstore;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::Fixture;
    use async_trait::async_trait;
    use byte_unit::Byte;
    use cryfs_blockstore::{InMemoryBlockStore, LockingBlockStore};
    use cryfs_utils::async_drop::AsyncDropGuard;

    struct TestFixture<const BLOCK_SIZE_BYTES: u64>;
    #[async_trait]
    impl<const BLOCK_SIZE_BYTES: u64> Fixture for TestFixture<BLOCK_SIZE_BYTES> {
        type ConcreteBlobStore = BlobStoreOnBlocks<InMemoryBlockStore>;
        fn new() -> Self {
            Self {}
        }
        async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlobStore> {
            BlobStoreOnBlocks::new(
                LockingBlockStore::new(InMemoryBlockStore::new()),
                Byte::from_u64(BLOCK_SIZE_BYTES),
            )
            .await
            .unwrap()
        }
        async fn yield_fixture(&self, _store: &Self::ConcreteBlobStore) {}
    }

    crate::instantiate_blobstore_tests!(TestFixture<1024>, (flavor = "multi_thread"));
}
