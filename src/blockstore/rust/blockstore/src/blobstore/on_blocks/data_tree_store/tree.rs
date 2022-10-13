use anyhow::{anyhow, bail, ensure, Result};
use async_recursion::async_recursion;
use async_trait::async_trait;
use divrem::DivCeil;
use std::fmt::{self, Debug};
use std::marker::PhantomData;
use std::num::{NonZeroU32, NonZeroU64};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use super::size_cache::SizeCache;
use super::traversal::{self, LeafHandle};
use crate::blobstore::on_blocks::data_node_store::{
    DataInnerNode, DataNode, DataNodeStore, NodeLayout, RemoveResult,
};
use crate::blockstore::{low_level::BlockStore, BlockId};
use crate::data::Data;
use crate::utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};
use crate::utils::stream::for_each_unordered;

pub struct DataTree<B: BlockStore + Send + Sync> {
    // The lock on the root node also ensures that there never are two [DataTree] instances for the same tree
    // &mut self in all the methods makes sure we don't run into race conditions where
    // one task modifies a tree we're currently trying to read somewhere else.
    // TODO Think about whether we can allow some kind of concurrency, e.g. multiple concurrent reads
    // (but we may have to think about how that interacts with the size_cache since even reads might write to that)

    // root_node is always some except in the middle of computations
    root_node: Option<DataNode<B>>,
    node_store: AsyncDropGuard<AsyncDropArc<DataNodeStore<B>>>,

    // TODO Think through all operations and whether they can change data that is cached in num_bytes_cache. Update cache if necessary.
    //      num_bytes_cache caches a bit differently than the C++ cache did.
    num_bytes_cache: SizeCache,
}

impl<B: BlockStore + Send + Sync> DataTree<B> {
    pub fn new(
        root_node: DataNode<B>,
        node_store: AsyncDropGuard<AsyncDropArc<DataNodeStore<B>>>,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            root_node: Some(root_node),
            node_store,
            num_bytes_cache: SizeCache::SizeUnknown,
        })
    }

    pub async fn num_bytes(&mut self) -> Result<u64> {
        self.num_bytes_cache
            .get_or_calculate_num_bytes(
                &self.node_store,
                &self.root_node.as_ref().expect("root_node is None"),
            )
            .await
    }

    pub async fn num_nodes(&mut self) -> Result<u64> {
        let root_node = self.root_node.as_ref().expect("root_node is None");
        let mut num_nodes_current_level = self
            .num_bytes_cache
            .get_or_calculate_num_leaves(&self.node_store, root_node)
            .await?
            .get();
        let mut total_num_nodes = num_nodes_current_level;
        for level in 0..root_node.depth() {
            num_nodes_current_level = num_nodes_current_level.div_ceil(u64::from(
                self.node_store.layout().max_children_per_inner_node(),
            ));
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
        let root = self.root_node
            .as_mut()
            .expect("root_node is None");
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
                is_right_border_leaf: bool,
                mut leaf: LeafHandle<'_, B>,
            ) -> Result<()> {
                assert_eq!(self.new_num_leaves.get() - 1, index);
                // TODO Does the following assertion make sense? assert!(is_right_border_leaf);
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
                let needed_nodes_on_child_level = self
                    .new_num_leaves
                    .get()
                    .div_ceil(max_leaves_per_child.get());
                let needed_nodes_on_same_level = needed_nodes_on_child_level.div_ceil(u64::from(
                    self.node_store.layout().max_children_per_inner_node(),
                ));
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
                    DataTree::_remove_subtree_by_root_id(
                        &self.node_store,
                        depth.get() - 1,
                        block_id,
                    )
                    .await
                })
                .await?;

                Ok(())
            }
        }

        let max_bytes_per_leaf = u64::from(self.node_store.layout().max_bytes_per_leaf());
        let new_num_leaves =
            NonZeroU64::new(new_num_bytes.div_ceil(max_bytes_per_leaf).max(1)).unwrap();
        let new_last_leaf_size =
            u32::try_from(new_num_bytes - (new_num_leaves.get() - 1) * max_bytes_per_leaf).unwrap();

        let root_node = self.root_node.take().expect("root_node is None");
        let new_root = self
            ._traverse_leaves_by_leaf_indices_return_new_root::<Callbacks<'_, B>, true>(
                root_node,
                new_num_leaves.get() - 1,
                new_num_leaves.get(),
                &Callbacks {
                    node_store: &self.node_store,
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

    pub async fn remove(mut this: AsyncDropGuard<Self>) -> Result<()> {
        let root_node = this.root_node.take().expect("DataTree.root_node is None");
        Self::_remove_subtree(&this.node_store, root_node).await?;
        this.async_drop().await?;
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
            &self.node_store,
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

#[async_trait]
impl<B: BlockStore + Send + Sync> AsyncDrop for DataTree<B> {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.node_store.async_drop().await
    }
}

impl<B: BlockStore + Send + Sync> Debug for DataTree<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DataTree")
    }
}
