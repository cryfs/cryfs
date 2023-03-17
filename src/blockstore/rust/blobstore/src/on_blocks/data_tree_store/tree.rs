use anyhow::{anyhow, bail, ensure, Result};
use async_recursion::async_recursion;
use async_trait::async_trait;
use divrem::DivCeil;
use futures::{
    future::{self, FutureExt},
    stream::{self, Stream, StreamExt},
};
use std::fmt::{self, Debug};
use std::marker::PhantomData;
use std::num::{NonZeroU32, NonZeroU64};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use super::size_cache::SizeCache;
use super::traversal::{self, LeafHandle};
use crate::{
    on_blocks::data_node_store::{DataInnerNode, DataNode, DataNodeStore, NodeLayout},
    RemoveResult,
};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_utils::{data::Data, stream::for_each_unordered};

pub struct DataTree<'a, B: BlockStore + Send + Sync> {
    // The lock on the root node also ensures that there never are two [DataTree] instances for the same tree
    // &mut self in all the methods makes sure we don't run into race conditions where
    // one task modifies a tree we're currently trying to read somewhere else.
    // TODO Think about whether we can allow some kind of concurrency, e.g. multiple concurrent reads
    // (but we may have to think about how that interacts with the size_cache since even reads might write to that)

    // root_node is always some except in the middle of computations
    root_node: Option<DataNode<B>>,
    node_store: &'a DataNodeStore<B>,

    // TODO Think through all operations and whether they can change data that is cached in num_bytes_cache. Update cache if necessary.
    //      num_bytes_cache caches a bit differently than the C++ cache did.
    num_bytes_cache: SizeCache,
}

impl<'a, B: BlockStore + Send + Sync> DataTree<'a, B> {
    pub fn new(root_node: DataNode<B>, node_store: &'a DataNodeStore<B>) -> Self {
        Self {
            root_node: Some(root_node),
            node_store,
            num_bytes_cache: SizeCache::SizeUnknown,
        }
    }

    pub async fn num_bytes(&mut self) -> Result<u64> {
        self.num_bytes_cache
            .get_or_calculate_num_bytes(
                self.node_store,
                self.root_node.as_ref().expect("root_node is None"),
            )
            .await
    }

    pub async fn num_nodes(&mut self) -> Result<u64> {
        let root_node = self.root_node.as_ref().expect("root_node is None");
        let mut num_nodes_current_level = self
            .num_bytes_cache
            .get_or_calculate_num_leaves(self.node_store, root_node)
            .await?
            .get();
        let mut total_num_nodes = num_nodes_current_level;
        for _level in 0..root_node.depth() {
            num_nodes_current_level = DivCeil::div_ceil(
                num_nodes_current_level,
                u64::from(self.node_store.layout().max_children_per_inner_node()),
            );
            total_num_nodes += num_nodes_current_level;
        }
        Ok(total_num_nodes)
    }

    pub fn root_node_id(&self) -> &BlockId {
        self.root_node
            .as_ref()
            .expect("root_node is None")
            .block_id()
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
                        && index_of_first_leaf_byte + u64::from(leaf_data_offset) - self.offset
                            <= u64::try_from(target.len()).unwrap()
                        && index_of_first_leaf_byte
                            + u64::from(leaf_data_offset)
                            + u64::from(leaf_data_size)
                            - self.offset
                            <= u64::try_from(target.len()).unwrap(),
                    "Writing to target out of bounds: index_of_first_leaf_byte={}, offset={}, leaf_data_offset={}, leaf_data_size={}, target.len={}", index_of_first_leaf_byte, self.offset, leaf_data_offset, leaf_data_size, target.len(),
                );
                // TODO Simplify formula, make it easier to understand
                let target_begin =
                    index_of_first_leaf_byte + u64::from(leaf_data_offset) - self.offset;
                let target_end = target_begin + u64::from(leaf_data_size);
                let actual_target = &mut target
                    [usize::try_from(target_begin).unwrap()..usize::try_from(target_end).unwrap()];
                let actual_source = &leaf.data()[usize::try_from(leaf_data_offset).unwrap()
                    ..usize::try_from(leaf_data_offset + leaf_data_size).unwrap()];
                actual_target.copy_from_slice(actual_source);
                Ok(())
            }
            fn on_create_leaf(&self, _begin_byte: u64, _num_bytes: u32) -> Data {
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

    pub async fn read_all(&mut self) -> Result<Data> {
        //TODO Querying num_bytes can be inefficient. Is this possible without a call to num_bytes()?
        let num_bytes = self.num_bytes().await?;
        let mut result = Data::from(vec![0; usize::try_from(num_bytes).unwrap()]); // TODO Don't initialize with zero?
        self._do_read_bytes(0, &mut result).await?;
        Ok(result)
    }

    pub async fn write_bytes(&mut self, source: &[u8], offset: u64) -> Result<()> {
        struct Callbacks<'a> {
            layout: NodeLayout,
            offset: u64,
            source: &'a [u8],
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
                assert!(
                    index_of_first_leaf_byte + u64::from(leaf_data_offset) >= self.offset
                        && index_of_first_leaf_byte + u64::from(leaf_data_offset) - self.offset
                            <= u64::try_from(self.source.len()).unwrap()
                        && index_of_first_leaf_byte
                            + u64::from(leaf_data_offset)
                            + u64::from(leaf_data_size)
                            - self.offset
                            <= u64::try_from(self.source.len()).unwrap(),
                    "Reading from source out of bounds"
                );
                let source_begin =
                    index_of_first_leaf_byte + u64::from(leaf_data_offset) - self.offset;
                let source_end = source_begin + u64::from(leaf_data_size);
                let actual_source = &self.source
                    [usize::try_from(source_begin).unwrap()..usize::try_from(source_end).unwrap()];
                if leaf_data_offset == 0 && leaf_data_size == self.layout.max_bytes_per_leaf() {
                    leaf.overwrite_data(actual_source).await?;
                } else {
                    let actual_target =
                        &mut leaf.node().await?.data_mut()[usize::try_from(leaf_data_offset)
                            .unwrap()
                            ..usize::try_from(leaf_data_offset + leaf_data_size).unwrap()];
                    actual_target.copy_from_slice(actual_source);
                }
                Ok(())
            }
            fn on_create_leaf(&self, begin_byte: u64, num_bytes: u32) -> Data {
                assert!(
                    begin_byte >= self.offset
                        && begin_byte - self.offset <= u64::try_from(self.source.len()).unwrap()
                        && begin_byte - self.offset + u64::from(num_bytes)
                            <= u64::try_from(self.source.len()).unwrap(),
                    "Reading from source out of bounds"
                );
                // TODO Should we just return a borrowed slice from on_create_leaf instead of allocating a data object? Here and in other on_create_leaf instances?
                let mut data = Data::from(vec![0; usize::try_from(num_bytes).unwrap()]); // TODO Possible without zeroing out?
                let source_begin = begin_byte - self.offset;
                let source_end = source_begin + u64::from(num_bytes);
                let actual_source = &self.source
                    [usize::try_from(source_begin).unwrap()..usize::try_from(source_end).unwrap()];
                data.as_mut().copy_from_slice(actual_source);
                data
            }
        }

        self._traverse_leaves_by_byte_indices::<Callbacks, true>(
            offset,
            u64::try_from(source.len()).unwrap(),
            &Callbacks {
                layout: *self.node_store.layout(),
                offset,
                source,
            },
        )
        .await
    }

    pub async fn flush(&mut self) -> Result<()> {
        // TODO This doesn't actually flush the whole tree And I have to double check that this actually flushes the node to disk and doesn't just
        // write it into the cache. I might have to find a different solution here.
        let root = self.root_node.as_mut().expect("root_node is None");
        self.node_store.flush_node(root).await
    }

    pub async fn resize_num_bytes(&mut self, new_num_bytes: u64) -> Result<()> {
        struct Callbacks<'a, B: BlockStore + Send + Sync> {
            node_store: &'a DataNodeStore<B>,
            new_num_leaves: NonZeroU64,
            new_last_leaf_size: u32,
        }
        #[async_trait]
        impl<'a, B: BlockStore + Send + Sync> traversal::TraversalCallbacks<B> for Callbacks<'a, B> {
            async fn on_existing_leaf(
                &self,
                index: u64,
                _is_right_border_leaf: bool,
                mut leaf: LeafHandle<'_, B>,
            ) -> Result<()> {
                assert_eq!(self.new_num_leaves.get() - 1, index);
                // TODO Does the following assertion make sense?
                // assert!(
                //     is_right_border_leaf,
                //     "This should only be called for right border leaves"
                // );
                // This is only called if the new last leaf was already existing
                let leaf = leaf.node().await?;
                if leaf.num_bytes() != self.new_last_leaf_size {
                    leaf.resize(self.new_last_leaf_size);
                }
                Ok(())
            }
            fn on_create_leaf(&self, index: u64) -> Data {
                assert_eq!(self.new_num_leaves.get() - 1, index);
                // This is only called, if the new last leaf was not existing yet
                Data::from(vec![0; usize::try_from(self.new_last_leaf_size).unwrap()])
            }
            async fn on_backtrack_from_subtree(&self, node: &mut DataInnerNode<B>) -> Result<()> {
                // This is only called for the right border nodes of the new tree.
                // When growing size, the following is a no-op. When shrinking, we're deleting the children that aren't needed anymore.

                let max_leaves_per_child = self
                    .node_store
                    .layout()
                    .num_leaves_per_full_subtree(node.depth().get() - 1)?;
                let needed_nodes_on_child_level =
                    DivCeil::div_ceil(self.new_num_leaves.get(), max_leaves_per_child.get());
                let needed_nodes_on_same_level = DivCeil::div_ceil(
                    needed_nodes_on_child_level,
                    u64::from(self.node_store.layout().max_children_per_inner_node()),
                );
                let child_level_nodes_covered_by_siblings = (needed_nodes_on_same_level - 1)
                    * u64::from(self.node_store.layout().max_children_per_inner_node());
                let needed_children_for_right_border_node = u32::try_from(
                    needed_nodes_on_child_level - child_level_nodes_covered_by_siblings,
                )
                .unwrap();
                let children = node.children();
                assert!(
                    needed_children_for_right_border_node <= u32::try_from(children.len()).unwrap(),
                    "Node has too few children"
                );
                // All children to the right of the new right-border-node are removed including their subtree.
                let children_to_delete: Vec<BlockId> = children
                    .skip(usize::try_from(needed_children_for_right_border_node).unwrap())
                    .collect();
                let depth = node.depth();

                // Ordering: First remove the child block ids from the node, then remove the actual blocks.
                // This has a higher chance of keeping the file system in a consistent state if there's a power loss in the middle.
                node.shrink_num_children(
                    NonZeroU32::new(needed_children_for_right_border_node).unwrap(),
                )?;
                for_each_unordered(children_to_delete.into_iter(), move |block_id| async move {
                    DataTree::_remove_subtree_by_root_id(self.node_store, depth.get() - 1, block_id)
                        .await
                })
                .await?;

                Ok(())
            }
        }

        let max_bytes_per_leaf = u64::from(self.node_store.layout().max_bytes_per_leaf());
        let new_num_leaves =
            NonZeroU64::new(DivCeil::div_ceil(new_num_bytes, max_bytes_per_leaf).max(1)).unwrap();
        let new_last_leaf_size =
            u32::try_from(new_num_bytes - (new_num_leaves.get() - 1) * max_bytes_per_leaf).unwrap();

        let root_node = self.root_node.take().expect("root_node is None");
        let new_root = self
            ._traverse_leaves_by_leaf_indices_return_new_root::<Callbacks<'_, B>, true>(
                root_node,
                new_num_leaves.get() - 1,
                new_num_leaves.get(),
                &Callbacks {
                    node_store: self.node_store,
                    new_last_leaf_size,
                    new_num_leaves,
                },
            )
            .await?;
        self.root_node = Some(new_root);

        self.num_bytes_cache
            .update(self.node_store.layout(), new_num_leaves, new_num_bytes)?;
        Ok(())
    }

    pub async fn remove(mut self) -> Result<()> {
        let root_node = self.root_node.take().expect("DataTree.root_node is None");
        Self::_remove_subtree(self.node_store, root_node).await?;
        Ok(())
    }

    async fn _traverse_leaves_by_leaf_indices_return_new_root<
        C: traversal::TraversalCallbacks<B> + Sync,
        const ALLOW_WRITES: bool,
    >(
        &self,
        root_node: DataNode<B>,
        begin_index: u64,
        end_index: u64,
        callbacks: &C,
    ) -> Result<DataNode<B>> {
        if end_index <= begin_index {
            return Ok(root_node);
        }

        traversal::traverse_and_return_new_root::<B, C, ALLOW_WRITES>(
            self.node_store,
            root_node,
            begin_index,
            end_index,
            callbacks,
        )
        .await
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
        let end_leaf = DivCeil::div_ceil(end_byte, max_bytes_per_leaf);
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
            async fn on_backtrack_from_subtree(&self, _node: &mut DataInnerNode<B>) -> Result<()> {
                // do nothing
                Ok(())
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

        let root_node = self.root_node.take().expect("self.root_node is None");
        let new_root = self._traverse_leaves_by_leaf_indices_return_new_root::<WrappedCallbacks<'_, B, C, ALLOW_WRITES>, ALLOW_WRITES>(
            root_node,
            first_leaf,
            end_leaf,
            &wrapped_callbacks,
        )
        .await?;
        self.root_node = Some(new_root);
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

    async fn _remove_subtree(node_store: &DataNodeStore<B>, root: DataNode<B>) -> Result<()> {
        match root {
            DataNode::Leaf(_) => {
                root.remove(node_store).await?;
                Ok(())
            }
            DataNode::Inner(root) => {
                Self::_remove_subtree_of_inner_node(node_store, root).await?;
                Ok(())
            }
        }
    }

    async fn _remove_subtree_of_inner_node(
        node_store: &DataNodeStore<B>,
        root: DataInnerNode<B>,
    ) -> Result<()> {
        // Ordering: First remove the node itself, then remove the children.
        // This has a higher chance of keeping the file system in a consistent state if there's a power loss in the middle.
        let children: Vec<_> = root.children().collect();
        let depth = root.depth().get();
        root.upcast().remove(node_store).await?;
        for_each_unordered(children.into_iter(), |child_block_id| {
            Self::_remove_subtree_by_root_id(node_store, depth - 1, child_block_id)
        })
        .await?;
        Ok(())
    }

    #[async_recursion]
    async fn _remove_subtree_by_root_id(
        node_store: &DataNodeStore<B>,
        depth: u8,
        block_id: BlockId,
    ) -> Result<()> {
        if depth == 0 {
            // Here, we can remove the leaf node without even loading it
            let remove_result = node_store.remove_by_id(&block_id).await?;
            ensure!(
                RemoveResult::SuccessfullyRemoved == remove_result,
                "Tried to remove {:?} but didn't find it",
                block_id
            );
        } else {
            match node_store.load(block_id).await? {
                None => bail!(
                    "Tried to load inner node {:?} for removal but didn't find it",
                    block_id
                ),
                Some(DataNode::Leaf(_)) => bail!(
                    "Tried to load inner node {:?} for removal but it was a leaf",
                    block_id
                ),
                Some(DataNode::Inner(node)) => {
                    ensure!(
                        node.depth().get() == depth,
                        "Tried to load inner node {:?} at depth {} for removal but it had depth {}",
                        block_id,
                        depth,
                        node.depth()
                    );
                    Self::_remove_subtree_of_inner_node(node_store, node).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn all_blocks(&self) -> Result<Box<dyn Stream<Item = Result<BlockId>> + Unpin + '_>> {
        let root_node = self.root_node.as_ref().expect("root_node is None");
        self._all_blocks_in_subtree(root_node).await
    }

    async fn _all_blocks_in_subtree(
        &self,
        subtree_root: &DataNode<B>,
    ) -> Result<Box<dyn Stream<Item = Result<BlockId>> + Unpin + '_>> {
        match subtree_root {
            DataNode::Leaf(leaf) => Ok(Box::new(stream::once(future::ready(Ok(*leaf.block_id()))))),
            DataNode::Inner(inner) => {
                let block_ids_in_descendants = self._all_blocks_descendants_of(inner).await?;
                Ok(Box::new(
                    stream::once(future::ready(Ok(*inner.block_id())))
                        .chain(block_ids_in_descendants),
                ))
            }
        }
    }

    async fn _all_blocks_descendants_of(
        &self,
        subtree_root: &DataInnerNode<B>,
    ) -> Result<Box<dyn Stream<Item = Result<BlockId>> + Unpin + '_>> {
        // iter<stream<result<block_id>>>
        let subtree_stream = subtree_root.children().map(|child_id| {
            let child_stream = async move {
                self._all_blocks_in_subtree_of_id(child_id)
                    .await
                    // Transform Result<Stream<Result<BlockId>>> into Stream<Result<BlockId>>
                    .unwrap_or_else(|err| Box::new(stream::once(future::ready(Err(err)))))
            };
            // Transform Future<Stream<Result<BlockId>>> into Stream<Result<BlockId>>
            Box::pin(child_stream.flatten_stream())
        });
        let subtree_stream = stream::select_all(subtree_stream);
        Ok(Box::new(subtree_stream))
    }

    async fn _all_blocks_in_subtree_of_id(
        &self,
        subtree_root_id: BlockId,
    ) -> Result<Box<dyn Stream<Item = Result<BlockId>> + Unpin + '_>> {
        let child = self
            .node_store
            .load(subtree_root_id)
            .await?
            .ok_or_else(|| anyhow!("Didn't find block {:?}", subtree_root_id))?;
        self._all_blocks_in_subtree(&child).await
    }
}

#[async_trait]
trait TraversalByByteIndicesCallbacks<B: BlockStore + Send + Sync> {
    // TODO begin/count u32 or u64?
    async fn on_existing_leaf(
        &self,
        index_of_first_leaf_byte: u64,
        leaf: LeafHandle<'_, B>,
        leaf_data_offset: u32,
        leaf_data_size: u32,
    ) -> Result<()>;
    // TODO num_bytes u32 or u64?
    fn on_create_leaf(&self, begin_byte: u64, num_bytes: u32) -> Data;
}

impl<'a, B: BlockStore + Send + Sync> Debug for DataTree<'a, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DataTree")
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::data_node_store::NodeLayout;
    #[cfg(any(feature = "slow-tests-1", feature = "slow-tests-2",))]
    use super::super::super::data_tree_store::DataTree;
    use super::super::testutils::*;
    use cryfs_blockstore::BlockId;
    #[cfg(feature = "slow-tests-any")]
    use cryfs_blockstore::BlockStore;
    #[cfg(feature = "slow-tests-any")]
    use cryfs_utils::testutils::data_fixture::DataFixture;
    #[cfg(feature = "slow-tests-1")]
    use divrem::DivCeil;
    #[cfg(feature = "slow-tests-any")]
    use rstest::rstest;
    #[cfg(feature = "slow-tests-any")]
    use rstest_reuse::{apply, template};

    #[cfg(feature = "slow-tests-any")]
    mod testutils {
        use super::super::super::super::data_node_store::DataNodeStore;
        use super::*;

        #[derive(Clone, Copy)]
        pub enum ParamNum {
            Val(u64),
            MaxBytesPerLeafMinus(u64),
            HalfMaxBytesPerLeaf,
            MaxChildrenPerInnerNodeMinus(u64),
            NumFullLeavesForThreeLevelTreeWithLastInnerHasOneChild,
            NumFullLeavesForThreeLevelTreeWithLastInnerHasHalfNumChildren,
            NumFullLeavesForThreeLevelTree,
            NumFullLeavesForFourLevelMinDataTree,
        }
        impl ParamNum {
            pub fn eval(&self, layout: NodeLayout) -> u64 {
                match self {
                    Self::Val(val) => *val,
                    Self::MaxBytesPerLeafMinus(val) => layout.max_bytes_per_leaf() as u64 - val,
                    Self::HalfMaxBytesPerLeaf => layout.max_bytes_per_leaf() as u64 / 2,
                    Self::MaxChildrenPerInnerNodeMinus(val) => {
                        layout.max_children_per_inner_node() as u64 - val
                    }
                    Self::NumFullLeavesForThreeLevelTreeWithLastInnerHasOneChild => {
                        layout.max_children_per_inner_node() as u64
                            * (layout.max_children_per_inner_node() as u64 - 1)
                            + 1
                            - 1
                    }
                    Self::NumFullLeavesForThreeLevelTreeWithLastInnerHasHalfNumChildren => {
                        layout.max_children_per_inner_node() as u64
                            * (layout.max_children_per_inner_node() as u64 - 1)
                            + layout.max_children_per_inner_node() as u64 / 2
                            - 1
                    }
                    Self::NumFullLeavesForThreeLevelTree => {
                        layout.max_children_per_inner_node() as u64
                            * layout.max_children_per_inner_node() as u64
                            - 1
                    }
                    Self::NumFullLeavesForFourLevelMinDataTree => {
                        (layout.max_children_per_inner_node() as u64 - 1)
                            * layout.max_children_per_inner_node() as u64
                            * layout.max_children_per_inner_node() as u64
                    }
                }
            }
        }

        /// Parameter for creating a tree.
        /// This can be used for parameterized tests
        #[derive(Clone, Copy)]
        pub struct Parameter {
            /// Number of full leaves in the tree (the tree will have a total of [num_full_leaves] + 1 leaves)
            pub num_full_leaves: ParamNum,

            /// Number of bytes in the last leaf
            pub last_leaf_num_bytes: ParamNum,
        }
        impl Parameter {
            #[cfg(feature = "slow-tests-1")]
            pub fn expected_num_nodes(&self, layout: NodeLayout) -> u64 {
                let mut num_nodes = 0;
                let num_leaves = 1 + self.num_full_leaves.eval(layout);
                let mut num_nodes_current_level = num_leaves;
                while num_nodes_current_level > 1 {
                    num_nodes += num_nodes_current_level;
                    num_nodes_current_level = DivCeil::div_ceil(
                        num_nodes_current_level,
                        layout.max_children_per_inner_node() as u64,
                    );
                }
                assert!(num_nodes_current_level == 1);
                num_nodes += 1;
                num_nodes
            }

            #[cfg(any(
                feature = "slow-tests-1",
                feature = "slow-tests-2",
                feature = "slow-tests-4",
            ))]
            pub fn expected_num_leaves(&self, layout: NodeLayout) -> u64 {
                self.num_full_leaves.eval(layout) + 1
            }

            pub fn expected_num_bytes(&self, layout: NodeLayout) -> u64 {
                self.num_full_leaves.eval(layout) * layout.max_bytes_per_leaf() as u64
                    + self.last_leaf_num_bytes.eval(layout)
            }

            #[cfg(feature = "slow-tests-1")]
            pub async fn create_tree<B: BlockStore + Send + Sync>(
                &self,
                nodestore: &DataNodeStore<B>,
            ) -> BlockId {
                let id = manually_create_tree(
                    nodestore,
                    self.num_full_leaves.eval(*nodestore.layout()),
                    self.last_leaf_num_bytes.eval(*nodestore.layout()),
                    |_offset, num_bytes| vec![0; num_bytes].into(),
                )
                .await;
                nodestore.clear_cache_slow().await.unwrap();
                id
            }

            pub async fn create_tree_with_data<B: BlockStore + Send + Sync>(
                &self,
                nodestore: &DataNodeStore<B>,
                data: &DataFixture,
            ) -> BlockId {
                let generate_leaf_data = |offset: u64, num_bytes: usize| {
                    let mut result = vec![0; num_bytes];
                    data.generate(offset, &mut result);
                    result.into()
                };
                let id = manually_create_tree(
                    nodestore,
                    self.num_full_leaves.eval(*nodestore.layout()),
                    self.last_leaf_num_bytes.eval(*nodestore.layout()),
                    generate_leaf_data,
                )
                .await;
                nodestore.clear_cache_slow().await.unwrap();
                id
            }
        }

        #[template]
        #[rstest]
        #[case::one_leaf_empty(Parameter {
                num_full_leaves: ParamNum::Val(0),
                last_leaf_num_bytes: ParamNum::Val(0),
            })]
        #[case::one_leaf_almost_empty(Parameter {
                num_full_leaves: ParamNum::Val(0),
                last_leaf_num_bytes: ParamNum::Val(1),
            })]
        #[case::one_leaf_half_full(Parameter {
                num_full_leaves: ParamNum::Val(0),
                last_leaf_num_bytes: ParamNum::HalfMaxBytesPerLeaf,
            })]
        #[case::one_leaf_full(Parameter {
                num_full_leaves: ParamNum::Val(0),
                last_leaf_num_bytes: ParamNum::MaxBytesPerLeafMinus(0),
            })]
        #[case::two_leaves_last_leaf_empty(Parameter {
                num_full_leaves: ParamNum::Val(1),
                last_leaf_num_bytes: ParamNum::Val(0),
            })]
        #[case::two_leaves_last_leaf_almost_empty(Parameter {
                num_full_leaves: ParamNum::Val(1),
                last_leaf_num_bytes: ParamNum::Val(1),
            })]
        #[case::two_leaves_last_leaf_half_full(Parameter {
                num_full_leaves: ParamNum::Val(1),
                last_leaf_num_bytes: ParamNum::HalfMaxBytesPerLeaf,
            })]
        #[case::two_leaves_last_leaf_full(Parameter {
                num_full_leaves: ParamNum::Val(1),
                last_leaf_num_bytes: ParamNum::MaxBytesPerLeafMinus(0),
            })]
        #[case::almost_full_two_level_tree_last_leaf_empty(Parameter {
                num_full_leaves: ParamNum::MaxChildrenPerInnerNodeMinus(2),
                last_leaf_num_bytes: ParamNum::Val(0),
            })]
        #[case::almost_full_two_level_tree_last_leaf_almost_empty(Parameter {
                num_full_leaves: ParamNum::MaxChildrenPerInnerNodeMinus(2),
                last_leaf_num_bytes: ParamNum::Val(1),
            })]
        #[case::almost_full_two_level_tree_last_leaf_half_full(Parameter {
                num_full_leaves: ParamNum::MaxChildrenPerInnerNodeMinus(2),
                last_leaf_num_bytes: ParamNum::HalfMaxBytesPerLeaf,
            })]
        #[case::almost_full_two_level_tree_last_leaf_full(Parameter {
                num_full_leaves: ParamNum::MaxChildrenPerInnerNodeMinus(2),
                last_leaf_num_bytes: ParamNum::MaxBytesPerLeafMinus(0),
            })]
        #[case::full_two_level_tree_last_leaf_empty(Parameter {
                num_full_leaves: ParamNum::MaxChildrenPerInnerNodeMinus(1),
                last_leaf_num_bytes: ParamNum::Val(0),
            })]
        #[case::full_two_level_tree_last_leaf_almost_empty(Parameter {
                num_full_leaves: ParamNum::MaxChildrenPerInnerNodeMinus(1),
                last_leaf_num_bytes: ParamNum::Val(1),
            })]
        #[case::full_two_level_tree_last_leaf_half_full(Parameter {
                num_full_leaves: ParamNum::MaxChildrenPerInnerNodeMinus(1),
                last_leaf_num_bytes: ParamNum::HalfMaxBytesPerLeaf,
            })]
        #[case::full_two_level_tree_last_leaf_full(Parameter {
                num_full_leaves: ParamNum::MaxChildrenPerInnerNodeMinus(1),
                last_leaf_num_bytes: ParamNum::MaxBytesPerLeafMinus(0),
            })]
        #[case::three_level_tree_last_inner_has_one_child_last_leaf_empty(Parameter {
                num_full_leaves: ParamNum::NumFullLeavesForThreeLevelTreeWithLastInnerHasOneChild,
                last_leaf_num_bytes: ParamNum::Val(0),
            })]
        #[case::three_level_tree_last_inner_has_one_child_last_leaf_almost_empty(Parameter {
                num_full_leaves: ParamNum::NumFullLeavesForThreeLevelTreeWithLastInnerHasOneChild,
                last_leaf_num_bytes: ParamNum::Val(1),
            })]
        #[case::three_level_tree_last_inner_has_one_child_last_leaf_half_full(Parameter {
                num_full_leaves: ParamNum::NumFullLeavesForThreeLevelTreeWithLastInnerHasOneChild,
                last_leaf_num_bytes: ParamNum::HalfMaxBytesPerLeaf,
            })]
        #[case::three_level_tree_last_inner_has_one_child_last_leaf_full(Parameter {
                num_full_leaves: ParamNum::NumFullLeavesForThreeLevelTreeWithLastInnerHasOneChild,
                last_leaf_num_bytes: ParamNum::MaxBytesPerLeafMinus(0),
            })]
        #[case::three_level_tree_last_inner_has_half_num_children_last_leaf_empty(Parameter {
                num_full_leaves: ParamNum::NumFullLeavesForThreeLevelTreeWithLastInnerHasHalfNumChildren,
                last_leaf_num_bytes: ParamNum::Val(0),
            })]
        #[case::three_level_tree_last_inner_has_half_num_children_last_leaf_almost_empty(Parameter {
                num_full_leaves: ParamNum::NumFullLeavesForThreeLevelTreeWithLastInnerHasHalfNumChildren,
                last_leaf_num_bytes: ParamNum::Val(1),
            })]
        #[case::three_level_tree_last_inner_has_half_num_children_last_leaf_half_full(Parameter {
                num_full_leaves: ParamNum::NumFullLeavesForThreeLevelTreeWithLastInnerHasHalfNumChildren,
                last_leaf_num_bytes: ParamNum::HalfMaxBytesPerLeaf,
            })]
        #[case::three_level_tree_last_inner_has_half_num_children_last_leaf_full(Parameter {
                num_full_leaves: ParamNum::NumFullLeavesForThreeLevelTreeWithLastInnerHasHalfNumChildren,
                last_leaf_num_bytes: ParamNum::MaxBytesPerLeafMinus(0),
            })]
        #[case::full_three_level_tree_last_leaf_empty(Parameter {
                num_full_leaves: ParamNum::NumFullLeavesForThreeLevelTree,
                last_leaf_num_bytes: ParamNum::Val(0),
            })]
        #[case::full_three_level_tree_last_leaf_almost_empty(Parameter {
                num_full_leaves: ParamNum::NumFullLeavesForThreeLevelTree,
                last_leaf_num_bytes: ParamNum::Val(1),
            })]
        #[case::full_three_level_tree_last_leaf_half_full(Parameter {
                num_full_leaves: ParamNum::NumFullLeavesForThreeLevelTree,
                last_leaf_num_bytes: ParamNum::HalfMaxBytesPerLeaf,
            })]
        #[case::full_three_level_tree_last_leaf_full(Parameter {
                num_full_leaves: ParamNum::NumFullLeavesForThreeLevelTree,
                last_leaf_num_bytes: ParamNum::MaxBytesPerLeafMinus(0),
            })]
        #[case::four_level_min_data_tree_last_leaf_empty(Parameter {
                num_full_leaves: ParamNum::NumFullLeavesForFourLevelMinDataTree,
                last_leaf_num_bytes: ParamNum::Val(0),
            })]
        #[case::four_level_min_data_tree_last_leaf_almost_empty(Parameter {
                num_full_leaves: ParamNum::NumFullLeavesForFourLevelMinDataTree,
                last_leaf_num_bytes: ParamNum::Val(1),
            })]
        #[case::four_level_min_data_tree_last_leaf_half_full(Parameter {
                num_full_leaves: ParamNum::NumFullLeavesForFourLevelMinDataTree,
                last_leaf_num_bytes: ParamNum::HalfMaxBytesPerLeaf,
            })]
        #[case::four_level_min_data_tree_last_leaf_full(Parameter {
                num_full_leaves: ParamNum::NumFullLeavesForFourLevelMinDataTree,
                last_leaf_num_bytes: ParamNum::MaxBytesPerLeafMinus(0),
            })]
        fn tree_parameters<Fn>(#[case] param: Parameter) {}

        #[cfg(any(
            feature = "slow-tests-1",
            feature = "slow-tests-2",
            feature = "slow-tests-4",
        ))]
        #[derive(Clone, Copy, Debug)]
        pub enum LeafIndex {
            FromStart(i64),
            FromMid(i64),
            FromEnd(i64),
        }
        #[cfg(any(
            feature = "slow-tests-1",
            feature = "slow-tests-2",
            feature = "slow-tests-4",
        ))]
        impl LeafIndex {
            pub fn get(&self, num_leaves: u64) -> u64 {
                match self {
                    LeafIndex::FromStart(i) => *i as u64,
                    LeafIndex::FromMid(i) => (num_leaves as i64 / 2 + i).max(0) as u64,
                    LeafIndex::FromEnd(i) => (num_leaves as i64 + i).max(0) as u64,
                }
            }
        }
    }

    mod num_bytes_and_num_nodes {
        #[cfg(feature = "slow-tests-1")]
        use super::testutils::*;
        use super::*;

        #[tokio::test]
        async fn new_tree() {
            with_treestore(|store| {
                Box::pin(async move {
                    let mut tree = store.create_tree().await.unwrap();
                    assert_eq!(0, tree.num_bytes().await.unwrap());
                    assert_eq!(1, tree.num_nodes().await.unwrap());
                })
            })
            .await;
        }

        #[tokio::test]
        async fn check_test_setup() {
            with_treestore(|store| {
                Box::pin(async move {
                    let layout = NodeLayout {
                        block_size_bytes: PHYSICAL_BLOCK_SIZE_BYTES,
                    };
                    // Just make sure our calculation of LAYOUT is correct
                    assert_eq!(
                        layout.max_bytes_per_leaf(),
                        store.virtual_block_size_bytes(),
                    );
                })
            })
            .await
        }

        #[cfg(feature = "slow-tests-1")]
        #[apply(super::testutils::tree_parameters)]
        #[tokio::test]
        async fn build_tree_via_resize_and_check_num_bytes_and_num_nodes(
            #[values(40, 64, 512)] block_size_bytes: u32,
            #[case] param: Parameter,
        ) {
            let layout = NodeLayout { block_size_bytes };
            if param.num_full_leaves.eval(layout) > 0 && param.last_leaf_num_bytes.eval(layout) == 0
            {
                // This is a special case where we can't build the tree via a call to [resize_num_bytes]
                // because that would never leave the last leaf empty
                return;
            }
            with_treestore_with_blocksize(block_size_bytes, move |store| {
                Box::pin(async move {
                    let mut tree = store.create_tree().await.unwrap();
                    let num_bytes = layout.max_bytes_per_leaf() as u64
                        * param.num_full_leaves.eval(layout)
                        + param.last_leaf_num_bytes.eval(layout);
                    tree.resize_num_bytes(num_bytes).await.unwrap();
                    assert_eq!(num_bytes, tree.num_bytes().await.unwrap());
                    assert_eq!(
                        param.expected_num_nodes(layout),
                        tree.num_nodes().await.unwrap()
                    );

                    // Check the values are still the same when queried again
                    // (they should now be returned from the cache instead of calculated)
                    assert_eq!(num_bytes, tree.num_bytes().await.unwrap());
                    assert_eq!(
                        param.expected_num_nodes(layout),
                        tree.num_nodes().await.unwrap()
                    );
                })
            })
            .await;
        }

        #[cfg(feature = "slow-tests-1")]
        #[apply(super::testutils::tree_parameters)]
        #[tokio::test]
        async fn build_tree_manually_and_check_num_bytes_and_num_nodes(
            #[values(40, 64, 512)] block_size_bytes: u32,
            #[case] param: Parameter,
        ) {
            let layout = NodeLayout { block_size_bytes };
            with_treestore_and_nodestore_with_blocksize(
                block_size_bytes,
                |treestore, nodestore| {
                    Box::pin(async move {
                        let root_id = param.create_tree(nodestore).await;

                        let mut tree = treestore.load_tree(root_id).await.unwrap().unwrap();
                        assert_eq!(
                            param.expected_num_bytes(layout),
                            tree.num_bytes().await.unwrap()
                        );
                        assert_eq!(
                            param.expected_num_nodes(layout),
                            tree.num_nodes().await.unwrap()
                        );

                        // Check the values are still the same when queried again
                        // (they should now be returned from the cache instead of calculated)
                        assert_eq!(
                            param.expected_num_bytes(layout),
                            tree.num_bytes().await.unwrap()
                        );
                        assert_eq!(
                            param.expected_num_nodes(layout),
                            tree.num_nodes().await.unwrap()
                        );
                    })
                },
            )
            .await;
        }
    }

    mod root_node_id {
        use super::*;

        #[tokio::test]
        async fn after_creating_one_node_tree() {
            with_treestore(|store| {
                Box::pin(async move {
                    let root_id = BlockId::from_hex("18834bc490faaab6bfdc6a53864cd0a8").unwrap();
                    let tree = store.try_create_tree(root_id).await.unwrap().unwrap();
                    assert_eq!(root_id, *tree.root_node_id());
                })
            })
            .await
        }

        #[tokio::test]
        async fn after_loading_one_node_tree() {
            with_treestore(|store| {
                Box::pin(async move {
                    let root_id = BlockId::from_hex("18834bc490faaab6bfdc6a53864cd0a8").unwrap();
                    store.try_create_tree(root_id).await.unwrap().unwrap();
                    let tree = store.load_tree(root_id).await.unwrap().unwrap();
                    assert_eq!(root_id, *tree.root_node_id());
                })
            })
            .await
        }

        #[tokio::test]
        async fn after_creating_multi_node_tree() {
            with_treestore(|store| {
                Box::pin(async move {
                    let root_id = BlockId::from_hex("18834bc490faaab6bfdc6a53864cd0a8").unwrap();
                    let mut tree = store.try_create_tree(root_id).await.unwrap().unwrap();
                    tree.resize_num_bytes(store.virtual_block_size_bytes() as u64 * 100)
                        .await
                        .unwrap();
                    assert_eq!(root_id, *tree.root_node_id());
                })
            })
            .await
        }

        #[tokio::test]
        async fn after_loading_multi_node_tree() {
            with_treestore(|store| {
                Box::pin(async move {
                    let root_id = BlockId::from_hex("18834bc490faaab6bfdc6a53864cd0a8").unwrap();
                    let mut tree = store.try_create_tree(root_id).await.unwrap().unwrap();
                    tree.resize_num_bytes(store.virtual_block_size_bytes() as u64 * 100)
                        .await
                        .unwrap();
                    std::mem::drop(tree);

                    let tree = store.load_tree(root_id).await.unwrap().unwrap();
                    assert_eq!(root_id, *tree.root_node_id());
                })
            })
            .await
        }
    }

    #[cfg(any(
        feature = "slow-tests-1",
        feature = "slow-tests-2",
        feature = "slow-tests-4",
    ))]
    macro_rules! instantiate_read_write_tests {
        ($test_fn:ident) => {
            #[apply(super::testutils::tree_parameters)]
            #[tokio::test]
            async fn whole_tree(
                #[values(40, 64, 512)] block_size_bytes: u32,
                #[case] param: Parameter,
            ) {
                let layout = NodeLayout { block_size_bytes };
                let expected_num_bytes = param.expected_num_bytes(layout);
                $test_fn(block_size_bytes, param, 0, expected_num_bytes as usize).await;
            }

            #[apply(super::testutils::tree_parameters)]
            #[tokio::test]
            async fn single_byte(
                #[case] param: Parameter,
                #[values(LeafIndex::FromStart(0), LeafIndex::FromStart(1), LeafIndex::FromMid(0), LeafIndex::FromEnd(-1), LeafIndex::FromEnd(0), LeafIndex::FromEnd(1))]
                leaf_index: LeafIndex,
                #[values(
                    ParamNum::Val(0),
                    ParamNum::Val(1),
                    ParamNum::HalfMaxBytesPerLeaf,
                    ParamNum::MaxBytesPerLeafMinus(2),
                    ParamNum::MaxBytesPerLeafMinus(1)
                )]
                byte_index_in_leaf: ParamNum,
            ) {
                // Using for-loop instead of #[values] because otherwise compile times go through the roof
                for block_size_bytes in [40, 64, 512] {
                    let layout = NodeLayout { block_size_bytes };
                    let leaf_index = leaf_index.get(param.expected_num_leaves(layout));
                    let byte_index = leaf_index * layout.max_bytes_per_leaf() as u64
                        + byte_index_in_leaf.eval(layout);
                    $test_fn(block_size_bytes, param, byte_index, 1).await;
                }
            }

            #[apply(super::testutils::tree_parameters)]
            #[tokio::test]
            async fn two_bytes(
                #[case] param: Parameter,
                #[values(LeafIndex::FromStart(0), LeafIndex::FromStart(1), LeafIndex::FromMid(0), LeafIndex::FromEnd(-1), LeafIndex::FromEnd(0), LeafIndex::FromEnd(1))]
                leaf_index: LeafIndex,
                // The last value of `LAYOUT.max_bytes_per_leaf() as u64 - 1` means we read across the leaf boundary
                #[values(
                    ParamNum::Val(0),
                    ParamNum::Val(1),
                    ParamNum::HalfMaxBytesPerLeaf,
                    ParamNum::MaxBytesPerLeafMinus(2),
                    ParamNum::MaxBytesPerLeafMinus(1)
                )]
                first_byte_index_in_leaf: ParamNum,
            ) {
                // Using for-loop instead of #[values] because otherwise compile times go through the roof
                for block_size_bytes in [40, 64, 512] {
                    let layout = NodeLayout { block_size_bytes };
                    let leaf_index = leaf_index.get(param.expected_num_leaves(layout));
                    let first_byte_index = leaf_index * layout.max_bytes_per_leaf() as u64
                        + first_byte_index_in_leaf.eval(layout);
                    $test_fn(block_size_bytes, param, first_byte_index, 2).await;
                }
            }

            #[apply(super::testutils::tree_parameters)]
            #[tokio::test]
            async fn single_leaf(
                #[case] param: Parameter,
                #[values(
                    LeafIndex::FromStart(0),
                    LeafIndex::FromMid(0),
                    LeafIndex::FromEnd(0),
                    LeafIndex::FromEnd(1)
                )]
                leaf_index: LeafIndex,
                #[values(
                    // Ranges starting at the beginning of the leaf
                    (ParamNum::Val(0), ParamNum::Val(0)), (ParamNum::Val(0), ParamNum::Val(1)), (ParamNum::Val(0), ParamNum::HalfMaxBytesPerLeaf), (ParamNum::Val(0), ParamNum::MaxBytesPerLeafMinus(2)), (ParamNum::Val(0), ParamNum::MaxBytesPerLeafMinus(1)), (ParamNum::Val(0), ParamNum::MaxBytesPerLeafMinus(0)),
                    // Ranges in the middle
                    (ParamNum::Val(1), ParamNum::Val(2)), (ParamNum::Val(2), ParamNum::Val(2)), (ParamNum::HalfMaxBytesPerLeaf, ParamNum::MaxBytesPerLeafMinus(1)),
                    // Ranges going until the end of the leaf
                    (ParamNum::Val(1), ParamNum::MaxBytesPerLeafMinus(0)), (ParamNum::HalfMaxBytesPerLeaf, ParamNum::MaxBytesPerLeafMinus(0)), (ParamNum::MaxBytesPerLeafMinus(1), ParamNum::MaxBytesPerLeafMinus(0)), (ParamNum::MaxBytesPerLeafMinus(0), ParamNum::MaxBytesPerLeafMinus(0))
                )]
                byte_indices: (ParamNum, ParamNum),
            ) {
                // Using for-loop instead of #[values] because otherwise compile times go through the roof
                for block_size_bytes in [40, 64, 512] {
                    let layout = NodeLayout { block_size_bytes };
                    let (begin_byte_index_in_leaf, end_byte_index_in_leaf) = byte_indices;
                    let first_leaf_byte = leaf_index.get(param.expected_num_leaves(layout))
                        * layout.max_bytes_per_leaf() as u64;
                    let begin_byte_index = first_leaf_byte + begin_byte_index_in_leaf.eval(layout);
                    let end_byte_index = first_leaf_byte + end_byte_index_in_leaf.eval(layout);
                    $test_fn(
                        block_size_bytes,
                        param,
                        begin_byte_index,
                        (end_byte_index - begin_byte_index) as usize,
                    )
                    .await;
                }
            }

            #[apply(super::testutils::tree_parameters)]
            #[tokio::test]
            async fn across_leaves(
                #[case] param: Parameter,
                #[values(
                    (LeafIndex::FromStart(0), LeafIndex::FromStart(1)),
                    (LeafIndex::FromStart(0), LeafIndex::FromMid(0)),
                    (LeafIndex::FromStart(0), LeafIndex::FromEnd(0)),
                    (LeafIndex::FromStart(0), LeafIndex::FromEnd(10)),
                    (LeafIndex::FromStart(10), LeafIndex::FromEnd(-10)),
                    (LeafIndex::FromMid(0), LeafIndex::FromMid(1)),
                    (LeafIndex::FromMid(0), LeafIndex::FromEnd(0)),
                    (LeafIndex::FromMid(0), LeafIndex::FromEnd(10)),
                    (LeafIndex::FromEnd(0), LeafIndex::FromEnd(5)),
                    (LeafIndex::FromEnd(1), LeafIndex::FromEnd(5)),
                    (LeafIndex::FromEnd(20), LeafIndex::FromEnd(50)),
                )]
                leaf_indices: (LeafIndex, LeafIndex),
                #[values(
                    ParamNum::Val(0),
                    ParamNum::HalfMaxBytesPerLeaf,
                    ParamNum::MaxBytesPerLeafMinus(1)
                )]
                begin_byte_index_in_leaf: ParamNum,
                #[values(
                    ParamNum::Val(1),
                    ParamNum::HalfMaxBytesPerLeaf,
                    ParamNum::MaxBytesPerLeafMinus(0)
                )]
                end_byte_index_in_leaf: ParamNum,
            ) {
                // Using for-loop instead of #[values] because otherwise compile times go through the roof
                for block_size_bytes in [40, 64, 512] {
                    let layout = NodeLayout { block_size_bytes };
                    let (begin_leaf_index, last_leaf_index) = leaf_indices;
                    let begin_byte_index = {
                        let first_leaf_byte = begin_leaf_index.get(param.expected_num_leaves(layout))
                            * layout.max_bytes_per_leaf() as u64;
                        first_leaf_byte + begin_byte_index_in_leaf.eval(layout)
                    };
                    let end_byte_index = {
                        let first_leaf_byte = last_leaf_index.get(param.expected_num_leaves(layout))
                            * layout.max_bytes_per_leaf() as u64;
                        first_leaf_byte + end_byte_index_in_leaf.eval(layout)
                    };
                    if end_byte_index < begin_byte_index {
                        return;
                    }
                    $test_fn(
                        block_size_bytes,
                        param,
                        begin_byte_index,
                        (end_byte_index - begin_byte_index) as usize,
                    )
                    .await;
                }
            }
        };
    }

    #[cfg(feature = "slow-tests-1")]
    mod read_bytes {
        use super::testutils::*;
        use super::*;

        async fn assert_reads_correct_data<'a, B: BlockStore + Send + Sync>(
            tree: &mut DataTree<'a, B>,
            data: &DataFixture,
            offset: u64,
            num_bytes: usize,
        ) {
            let mut read_data = vec![0; num_bytes];
            tree.read_bytes(offset, &mut read_data).await.unwrap();
            let mut expected_data = vec![0; num_bytes];
            data.generate(offset, &mut expected_data);
            assert_eq!(expected_data, read_data);
        }

        async fn assert_reading_is_out_of_range<'a, B: BlockStore + Send + Sync>(
            tree: &mut DataTree<'a, B>,
            layout: NodeLayout,
            params: &Parameter,
            offset: u64,
            num_bytes: usize,
        ) {
            assert_eq!(
                tree.read_bytes(offset, &mut vec![0; num_bytes])
                    .await
                    .unwrap_err()
                    .to_string(),
                format!("DataTree::read_bytes() tried to read range {}..{} but only has {} bytes stored. Use try_read_bytes() if this should be allowed.", offset, offset + num_bytes as u64, params.expected_num_bytes(layout)),
            );
        }

        async fn test_read_bytes(
            block_size_bytes: u32,
            param: Parameter,
            offset: u64,
            num_bytes: usize,
        ) {
            let layout = NodeLayout { block_size_bytes };
            let expected_num_bytes = param.expected_num_bytes(layout);
            with_treestore_and_nodestore_with_blocksize(
                block_size_bytes,
                |treestore, nodestore| {
                    Box::pin(async move {
                        let data = DataFixture::new(0);
                        let tree_id = param.create_tree_with_data(nodestore, &data).await;
                        let mut tree = treestore.load_tree(tree_id).await.unwrap().unwrap();

                        if offset + num_bytes as u64 > expected_num_bytes {
                            assert_reading_is_out_of_range(
                                &mut tree, layout, &param, offset, num_bytes,
                            )
                            .await;
                        } else {
                            assert_reads_correct_data(&mut tree, &data, offset, num_bytes).await;
                        }
                    })
                },
            )
            .await;
        }

        instantiate_read_write_tests!(test_read_bytes);
    }

    #[cfg(feature = "slow-tests-2")]
    mod try_read_bytes {
        use super::testutils::*;
        use super::*;

        async fn assert_reads_correct_data<'a, B: BlockStore + Send + Sync>(
            tree: &mut DataTree<'a, B>,
            data: &DataFixture,
            offset: u64,
            num_bytes: usize,
        ) {
            let mut read_data = vec![0; num_bytes];
            let num_read_bytes = tree.try_read_bytes(offset, &mut read_data).await.unwrap();
            assert_eq!(num_bytes, num_read_bytes);
            let mut expected_data = vec![0; num_bytes];
            data.generate(offset, &mut expected_data);
            assert_eq!(expected_data, read_data);
        }

        async fn assert_reading_is_out_of_range<'a, B: BlockStore + Send + Sync>(
            tree: &mut DataTree<'a, B>,
            layout: NodeLayout,
            data: &DataFixture,
            param: Parameter,
            offset: u64,
            num_bytes: usize,
        ) {
            let mut read_data = vec![0; num_bytes];
            let num_read_bytes = tree.try_read_bytes(offset, &mut read_data).await.unwrap();
            let expected_num_read_bytes = param
                .expected_num_bytes(layout)
                .saturating_sub(offset)
                .min(num_bytes as u64) as usize;
            assert_eq!(expected_num_read_bytes, num_read_bytes);
            let mut expected_data = vec![0; expected_num_read_bytes];
            data.generate(offset, &mut expected_data);
            assert_eq!(expected_data, &read_data[..expected_num_read_bytes]);
            assert_eq!(
                vec![0; num_bytes - expected_num_read_bytes],
                &read_data[expected_num_read_bytes..]
            );
        }

        async fn test_try_read_bytes(
            block_size_bytes: u32,
            param: Parameter,
            offset: u64,
            num_bytes: usize,
        ) {
            let layout = NodeLayout { block_size_bytes };
            with_treestore_and_nodestore_with_blocksize(
                block_size_bytes,
                |treestore, nodestore| {
                    Box::pin(async move {
                        let data = DataFixture::new(0);
                        let tree_id = param.create_tree_with_data(nodestore, &data).await;
                        let mut tree = treestore.load_tree(tree_id).await.unwrap().unwrap();
                        if offset + num_bytes as u64 > param.expected_num_bytes(layout) {
                            assert_reading_is_out_of_range(
                                &mut tree, layout, &data, param, offset, num_bytes,
                            )
                            .await;
                        } else {
                            assert_reads_correct_data(&mut tree, &data, offset, num_bytes).await;
                        }
                    })
                },
            )
            .await;
        }

        instantiate_read_write_tests!(test_try_read_bytes);
    }

    #[cfg(feature = "slow-tests-3")]
    mod read_all {
        use super::testutils::*;
        use super::*;
        use cryfs_utils::data::Data;

        #[apply(super::testutils::tree_parameters)]
        #[tokio::test]
        async fn read_whole_tree(
            #[values(40, 64, 512)] block_size_bytes: u32,
            #[case] param: Parameter,
        ) {
            let layout = NodeLayout { block_size_bytes };
            with_treestore_and_nodestore_with_blocksize(
                block_size_bytes,
                |treestore, nodestore| {
                    Box::pin(async move {
                        let data = DataFixture::new(0);
                        let tree_id = param.create_tree_with_data(nodestore, &data).await;
                        let mut tree = treestore.load_tree(tree_id).await.unwrap().unwrap();

                        let read_data = tree.read_all().await.unwrap();
                        assert_eq!(param.expected_num_bytes(layout) as usize, read_data.len());
                        let expected_data: Data =
                            data.get(param.expected_num_bytes(layout) as usize).into();
                        assert_eq!(expected_data, read_data);
                    })
                },
            )
            .await;
        }
    }

    #[cfg(feature = "slow-tests-4")]
    mod write_bytes {
        use super::testutils::*;
        use super::*;

        async fn test_write_bytes(
            block_size_bytes: u32,
            params: Parameter,
            offset: u64,
            num_bytes: usize,
        ) {
            let layout = NodeLayout { block_size_bytes };
            with_treestore_and_nodestore_with_blocksize(
                block_size_bytes,
                |treestore, nodestore| {
                    Box::pin(async move {
                        let base_data = DataFixture::new(0);
                        let write_data = DataFixture::new(1);

                        // Create tree with `base_data`
                        let tree_id = params.create_tree_with_data(nodestore, &base_data).await;
                        let mut tree = treestore.load_tree(tree_id).await.unwrap().unwrap();

                        // Write subregion with `write_data`
                        let source = write_data.get(num_bytes);
                        tree.write_bytes(&source, offset).await.unwrap();

                        // Read whole tree back so we can check it
                        let expected_new_size = if num_bytes == 0 {
                            // Writing doesn't grow the tree if num_bytes == 0, even if
                            // offset is beyond the current tree data size. So we don't
                            // max with offset+num_bytes here.
                            // TODO Is this actually the behavior we want? Or do we want write_bytes to grow the tree here?
                            params.expected_num_bytes(layout)
                        } else {
                            params
                                .expected_num_bytes(layout)
                                .max(offset + num_bytes as u64)
                        };
                        let read_data = tree.read_all().await.unwrap();
                        assert_eq!(expected_new_size, read_data.len() as u64);

                        // TODO Instead of checking the written data using a call to `read_all()`,
                        //      we should look at the actual node store (that's also how we're doing it in `read_bytes`
                        //      tests to write the initial tree data), make sure intermediate nodes are unchanged,
                        //      new intermediate nodes are added as needed, and leaf data is changed as needed.

                        // Now we expect the tree data to contain 4 sections:
                        // A) The data before the written subregion (up until the first of `offset` or of the old tree size)
                        //    This region should contain `base_data`.
                        // B) The data after the old data size up until `offset`. This only exists if our write started after the old data size.
                        //    This region should be zeroed out if it exists.
                        // C) The written subregion
                        //    This region should contain `write_data`.
                        // D) The data after the written subregion. This only exists if our write ended before the old data size
                        //    This region should contain `base_data` if it exists.

                        // Check section A
                        let section_a_end = offset.min(params.expected_num_bytes(layout)) as usize;
                        let expected_data = base_data.get(section_a_end);
                        assert_eq!(expected_data, &read_data[..section_a_end]);

                        if num_bytes != 0 {
                            // Check section B
                            let section_b_end = offset as usize;
                            let expected_data = vec![0; section_b_end - section_a_end];
                            assert_eq!(expected_data, &read_data[section_a_end..section_b_end]);

                            // Check section C
                            let section_c_end = (offset + num_bytes as u64) as usize;
                            let expected_data = write_data.get(num_bytes);
                            assert_eq!(expected_data, &read_data[section_b_end..section_c_end]);

                            // Check section D
                            let mut expected_data =
                                vec![0; (expected_new_size as usize - section_c_end) as usize];
                            base_data.generate(section_c_end as u64, &mut expected_data);
                            assert_eq!(expected_data, &read_data[section_c_end..]);
                        } else {
                            // See comment above, we don't grow the region if num_bytes == 0
                            // TODO See TODO above, is this actually the behavior we want?
                        }
                    })
                },
            )
            .await;
        }

        instantiate_read_write_tests!(test_write_bytes);
    }

    // TODO Test flush
    // TODO Test resize_num_bytes
    // TODO Test remove
    // TODO Test all_blocks
}
