mod block_store_adapter;

#[cfg(test)]
mod blockstore_tests {
    use super::*;
    mod with_flushing {
        use super::*;
        cryfs_blockstore::instantiate_lowlevel_blockstore_tests!(
            block_store_adapter::TestFixtureAdapter<
                true,
            >,
            (flavor = "multi_thread")
        );
    }
    mod without_flushing {
        use super::*;
        cryfs_blockstore::instantiate_lowlevel_blockstore_tests!(
            block_store_adapter::TestFixtureAdapter<
                false,
            >,
            (flavor = "multi_thread")
        );
    }
}
