mod block_store_adapter;

#[cfg(test)]
mod blockstore_tests {
    use super::*;

    mod block_size_minimal {
        use super::*;

        const MINIMAL_SIZE: u32 = crate::on_blocks::data_node_store::NodeLayout::header_len()
            as u32
            + 2 * crate::BLOBID_LEN as u32;

        mod with_flushing {
            use super::*;
            cryfs_blockstore::instantiate_lowlevel_blockstore_tests!(
                block_store_adapter::TestFixtureAdapter<
                    true, MINIMAL_SIZE,
                >,
                (flavor = "multi_thread")
            );
        }
        mod without_flushing {
            use super::*;
            cryfs_blockstore::instantiate_lowlevel_blockstore_tests!(
                block_store_adapter::TestFixtureAdapter<
                    false, MINIMAL_SIZE,
                >,
                (flavor = "multi_thread")
            );
        }
    }

    mod block_size_1kb {
        use super::*;
        mod with_flushing {
            use super::*;
            cryfs_blockstore::instantiate_lowlevel_blockstore_tests!(
                block_store_adapter::TestFixtureAdapter<
                    true, 1024,
                >,
                (flavor = "multi_thread")
            );
        }
        mod without_flushing {
            use super::*;
            cryfs_blockstore::instantiate_lowlevel_blockstore_tests!(
                block_store_adapter::TestFixtureAdapter<
                    false, 1024,
                >,
                (flavor = "multi_thread")
            );
        }
    }

    mod block_size_32kb {
        use super::*;
        mod with_flushing {
            use super::*;
            cryfs_blockstore::instantiate_lowlevel_blockstore_tests!(
                block_store_adapter::TestFixtureAdapter<
                    true, {32 * 1024},
                >,
                (flavor = "multi_thread")
            );
        }
        mod without_flushing {
            use super::*;
            cryfs_blockstore::instantiate_lowlevel_blockstore_tests!(
                block_store_adapter::TestFixtureAdapter<
                    false, {32 * 1024},
                >,
                (flavor = "multi_thread")
            );
        }
    }

    mod block_size_4mb {
        use super::*;
        mod with_flushing {
            use super::*;
            cryfs_blockstore::instantiate_lowlevel_blockstore_tests!(
                block_store_adapter::TestFixtureAdapter<
                    true, {4 * 1024 * 1024},
                >,
                (flavor = "multi_thread")
            );
        }
        mod without_flushing {
            use super::*;
            cryfs_blockstore::instantiate_lowlevel_blockstore_tests!(
                block_store_adapter::TestFixtureAdapter<
                    false, {4 * 1024 * 1024},
                >,
                (flavor = "multi_thread")
            );
        }
    }

    // TODO For these tests, we need to make sure that blockstore tests actually contain tests with large data amounts, otherwise we don't really test the tree structure.
}
