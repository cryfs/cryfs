mod data_node_store;
mod data_tree_store;

mod blob_on_blocks;
mod blobstore_on_blocks;

pub use blob_on_blocks::BlobOnBlocks;
pub use blobstore_on_blocks::BlobStoreOnBlocks;
pub use data_node_store::{DataInnerNode, DataLeafNode, DataNode, DataNodeStore};
pub use data_tree_store::{DataTree, DataTreeStore, LoadNodeError};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixture::Fixture;
    use async_trait::async_trait;
    use byte_unit::Byte;
    use cryfs_blockstore::{InMemoryBlockStore, LockingBlockStore};
    use cryfs_utils::async_drop::AsyncDropGuard;

    struct TestFixture<const BLOCK_SIZE_BYTES: u64>;
    #[async_trait]
    impl<const BLOCK_SIZE_BYTES: u64> Fixture for TestFixture<BLOCK_SIZE_BYTES> {
        type ConcreteBlobStore = BlobStoreOnBlocks<LockingBlockStore<InMemoryBlockStore>>;
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

    mod block_size_minimal {
        use super::*;

        const MINIMAL_SIZE: u64 = crate::on_blocks::data_node_store::NodeLayout::header_len()
            as u64
            + 2 * crate::BLOBID_LEN as u64;

        crate::instantiate_tests_for_blobstore!(
            TestFixture<MINIMAL_SIZE>,
            (flavor = "multi_thread")
        );
    }

    mod block_size_1kb {
        use super::*;
        crate::instantiate_tests_for_blobstore!(TestFixture<1024>, (flavor = "multi_thread"));
    }

    mod block_size_32kb {
        use super::*;
        crate::instantiate_tests_for_blobstore!(
            TestFixture<{ 32 * 1024 }>,
            (flavor = "multi_thread")
        );
    }

    mod block_size_4mb {
        use super::*;
        crate::instantiate_tests_for_blobstore!(
            TestFixture<{ 4 * 1024 * 1024 }>,
            (flavor = "multi_thread")
        );
    }

    // TODO For these tests, we need to make sure that blockstore tests actually contain tests with large data amounts, otherwise we don't really test the tree structure.
}
