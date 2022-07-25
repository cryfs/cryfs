use anyhow::{anyhow, ensure, Result};
use async_trait::async_trait;
use divrem::DivCeil;
use std::marker::PhantomData;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use super::size_cache::SizeCache;
use super::traversal::{self, LeafHandle};
use crate::blobstore::on_blocks::data_node_store::{
    DataInnerNode, DataNode, DataNodeStore, NodeLayout,
};
use crate::blockstore::low_level::BlockStore;
use crate::data::Data;
use crate::utils::async_drop::{AsyncDropArc, AsyncDropGuard};

pub struct DataTree<B: BlockStore + Send + Sync> {
    // The lock on the root node also ensures that there never are two [DataTree] instances for the same tree
    // &mut self in all the methods makes sure we don't run into race conditions where
    // one task modifies a tree we're currently trying to read somewhere else.
    // TODO Think about whether we can allow some kind of concurrency, e.g. multiple concurrent reads
    // (but we may have to think about how that interacts with the size_cache since even reads might write to that)

    // root_node is always some except in the middle of computations
    root_node: Option<DataNode<B>>,
    node_store: AsyncDropGuard<AsyncDropArc<DataNodeStore<B>>>,
    num_bytes_cache: SizeCache,
}

impl<B: BlockStore + Send + Sync> DataTree<B> {
    pub fn new(
        root_node: DataNode<B>,
        node_store: AsyncDropGuard<AsyncDropArc<DataNodeStore<B>>>,
    ) -> Self {
        Self {
            root_node: Some(root_node),
            node_store,
            num_bytes_cache: SizeCache::SizeUnknown,
        }
    }

    pub async fn num_bytes(&mut self) -> Result<u64> {
        self.num_bytes_cache
            .get_or_calculate_num_bytes(
                &self.node_store,
                &self.root_node.as_ref().expect("root_node is None"),
            )
            .await
    }

    // TODO Can we make read_bytes and try_read_bytes take &self instead of &mut self?
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
        struct Callbacks<'a> {
            offset: u64,
            target: Mutex<&'a mut [u8]>,
        }
        #[async_trait]
        impl<'a, B: BlockStore + Send + Sync> TraversalByByteIndicesCallbacks<B> for Callbacks<'a> {
            async fn on_existing_leaf(
                &self,
                index_of_first_leaf_byte: u64,
                mut leaf: LeafHandle<'_, B>,
                leaf_data_offset: u32,
                leaf_data_size: u32,
            ) -> Result<()> {
                let leaf = leaf.node().await?;
                let mut target = self.target.lock().unwrap();
                assert!(
                    index_of_first_leaf_byte + u64::from(leaf_data_offset) >= self.offset
                        && index_of_first_leaf_byte - self.offset + u64::from(leaf_data_offset)
                            <= u64::try_from(target.len()).unwrap()
                        && index_of_first_leaf_byte - self.offset
                            + u64::from(leaf_data_offset)
                            + u64::from(leaf_data_size)
                            <= u64::try_from(target.len()).unwrap(),
                    "Writing to target out of bounds"
                );
                // TODO Simplify formula, make it easier to understand
                let target_begin =
                    index_of_first_leaf_byte - self.offset + u64::from(leaf_data_offset);
                let target_end = target_begin + u64::from(leaf_data_size);
                let actual_target = &mut target
                    [usize::try_from(target_begin).unwrap()..usize::try_from(target_end).unwrap()];
                let actual_source = &leaf.data()[usize::try_from(leaf_data_offset).unwrap()
                    ..usize::try_from(leaf_data_offset + leaf_data_size).unwrap()];
                actual_target.copy_from_slice(actual_source);
                Ok(())
            }
            fn on_create_leaf(&self, _begin_byte: u64, _count: u32) -> Data {
                panic!("Reading shouldn't create new leaves");
            }
        }
        self._traverse_leaves_by_byte_indices::<Callbacks, false>(
            offset,
            u64::try_from(target.len()).unwrap(),
            &Callbacks {
                offset,
                target: Mutex::new(target),
            },
        )
        .await
    }

    async fn _traverse_leaves_by_leaf_indices<
        C: traversal::TraversalCallbacks<B> + Sync,
        const ALLOW_WRITES: bool,
    >(
        &mut self,
        begin_index: u64,
        end_index: u64,
        callbacks: &C,
    ) -> Result<()> {
        if end_index <= begin_index {
            return Ok(());
        }

        let new_root = traversal::traverse_and_return_new_root::<B, C, ALLOW_WRITES>(
            &self.node_store,
            self.root_node.take().expect("root_node is None"),
            begin_index,
            end_index,
            callbacks,
        )
        .await?;
        self.root_node = Some(new_root);
        Ok(())
    }

    async fn _traverse_leaves_by_byte_indices<
        C: TraversalByByteIndicesCallbacks<B> + Sync,
        const ALLOW_WRITES: bool,
    >(
        &mut self,
        begin_byte: u64,
        size_bytes: u64,
        callbacks: &C,
    ) -> Result<()> {
        if size_bytes == 0 {
            return Ok(());
        }

        let end_byte = begin_byte + size_bytes;
        let max_bytes_per_leaf = u64::from(self.node_store.layout().max_bytes_per_leaf());
        let first_leaf = begin_byte / max_bytes_per_leaf;
        let end_leaf = end_byte.div_ceil(max_bytes_per_leaf);
        struct WrappedCallbacks<
            'a,
            B: BlockStore + Send + Sync,
            C: TraversalByByteIndicesCallbacks<B>,
            const ALLOW_WRITES: bool,
        > {
            layout: NodeLayout,
            begin_byte: u64,
            end_byte: u64,
            first_leaf: u64,
            end_leaf: u64,
            blob_is_growing_from_this_traversal: AtomicBool,
            wrapped: &'a C,
            _b: PhantomData<B>,
        }
        #[async_trait]
        impl<
                'a,
                B: BlockStore + Send + Sync,
                C: TraversalByByteIndicesCallbacks<B> + Sync,
                const ALLOW_WRITES: bool,
            > traversal::TraversalCallbacks<B> for WrappedCallbacks<'a, B, C, ALLOW_WRITES>
        {
            async fn on_existing_leaf(
                &self,
                leaf_index: u64,
                is_right_border_leaf: bool,
                mut leaf_handle: LeafHandle<'_, B>,
            ) -> Result<()> {
                let max_bytes_per_leaf = u64::from(self.layout.max_bytes_per_leaf());
                let index_of_first_leaf_byte = leaf_index * max_bytes_per_leaf;
                assert!(
                    self.end_byte > index_of_first_leaf_byte,
                    "Traversal went to {} which is too far right for end_byte={}",
                    index_of_first_leaf_byte,
                    self.end_byte
                );
                let data_begin =
                    u32::try_from(self.begin_byte.saturating_sub(index_of_first_leaf_byte))
                        .unwrap();
                let data_end =
                    u32::try_from(max_bytes_per_leaf.min(self.end_byte - index_of_first_leaf_byte))
                        .unwrap();
                // If we are traversing exactly until the last leaf, then the last leaf wasn't resized by the traversal and might have a wrong size. We have to fix it.
                if is_right_border_leaf {
                    assert!(
                        leaf_index == self.end_leaf - 1,
                        "If we traversed further right, this wouldn't be the right border leaf."
                    );
                    let leaf = leaf_handle.node().await?;
                    if leaf.num_bytes() < data_end {
                        leaf.resize(data_end);
                        self.blob_is_growing_from_this_traversal
                            .store(true, Ordering::Relaxed);
                    }
                }
                self.wrapped
                    .on_existing_leaf(
                        index_of_first_leaf_byte,
                        leaf_handle,
                        data_begin,
                        data_end - data_begin,
                    )
                    .await?;
                Ok(())
            }
            fn on_create_leaf(&self, leaf_index: u64) -> Data {
                assert!(
                    ALLOW_WRITES,
                    "Cannot create leaves in a read-only traversal"
                );
                self.blob_is_growing_from_this_traversal
                    .store(true, Ordering::Relaxed);
                let max_bytes_per_leaf = u64::from(self.layout.max_bytes_per_leaf());
                let index_of_first_leaf_byte = leaf_index * max_bytes_per_leaf;
                assert!(
                    self.end_byte > index_of_first_leaf_byte,
                    "Traversal went to {} which is too far right for end_byte={}",
                    index_of_first_leaf_byte,
                    self.end_byte
                );
                let data_begin = self.begin_byte.saturating_sub(index_of_first_leaf_byte);
                let data_end = max_bytes_per_leaf.min(self.end_byte - index_of_first_leaf_byte);
                assert!(
                    leaf_index == self.first_leaf || data_begin == 0,
                    "Only the leftmost leaf can have a gap on the left"
                );
                assert!(
                    leaf_index == self.end_leaf - 1 || data_end == max_bytes_per_leaf,
                    "Only the rightmost leaf can have a gap on the right"
                );
                let mut data = self.wrapped.on_create_leaf(
                    index_of_first_leaf_byte + data_begin,
                    u32::try_from(data_end - data_begin).unwrap(),
                );
                assert!(
                    data.len() == usize::try_from(data_end - data_begin).unwrap(),
                    "Returned leaf data with {} bytes but expected {}",
                    data.len(),
                    data_end - data_begin
                );
                // If this leaf is created but only partly in the traversed region (i.e. dataBegin > leafBegin), we have to fill the data before the traversed region with zeroes.
                if data_begin != 0 {
                    data.grow_region(usize::try_from(data_begin).unwrap(), 0);
                }
                data
            }
            fn on_backtrack_from_subtree(&self, node: &mut DataInnerNode<B>) {
                // do nothing
            }
        }

        let wrapped_callbacks = WrappedCallbacks {
            layout: *self.node_store.layout(),
            begin_byte,
            end_byte,
            first_leaf,
            end_leaf,
            wrapped: callbacks,
            blob_is_growing_from_this_traversal: false.into(),
            _b: PhantomData,
        };

        self._traverse_leaves_by_leaf_indices::<WrappedCallbacks<'_, B, C, ALLOW_WRITES>, ALLOW_WRITES>(
            first_leaf,
            end_leaf,
            &wrapped_callbacks,
        )
        .await?;
        let blob_is_growing_from_this_traversal = wrapped_callbacks
            .blob_is_growing_from_this_traversal
            .load(Ordering::Relaxed);
        assert!(
            ALLOW_WRITES || !blob_is_growing_from_this_traversal,
            "Blob grew from traversal that didn't allow growing (i.e. reading)"
        );

        if blob_is_growing_from_this_traversal {
            let end_leaf = NonZeroU64::new(end_leaf)
                .expect("end_leaf cannot be zero because we checked above that size_bytes != 0");
            self.num_bytes_cache
                .update(self.node_store.layout(), end_leaf, end_byte)?;
        }

        Ok(())
    }
}

#[async_trait]
trait TraversalByByteIndicesCallbacks<B: BlockStore + Send + Sync> {
    // TODO begin/count u32 or u64?
    async fn on_existing_leaf(
        &self,
        index_of_first_leaf_byte: u64,
        leaf: LeafHandle<'_, B>,
        begin: u32,
        count: u32,
    ) -> Result<()>;
    // TODO count u32 or u64?
    fn on_create_leaf(&self, begin_byte: u64, count: u32) -> Data;
}
