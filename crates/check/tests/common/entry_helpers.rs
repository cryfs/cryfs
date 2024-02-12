use async_recursion::async_recursion;
use futures::{
    future::FutureExt,
    stream::{self, BoxStream, StreamExt},
};
use itertools::Itertools;
use rand::{rngs::SmallRng, seq::IteratorRandom, SeedableRng};
use std::fmt::Debug;
use std::time::SystemTime;

use cryfs_blobstore::{
    BlobId, BlobStore, DataInnerNode, DataLeafNode, DataNode, DataNodeStore, DataTree,
};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_check::BlobInfo;
use cryfs_cryfs::filesystem::fsblobstore::BlobType;
use cryfs_cryfs::{
    filesystem::fsblobstore::{DirBlob, FileBlob, FsBlob, FsBlobStore, SymlinkBlob},
    utils::fs_types::{Gid, Mode, Uid},
};
use cryfs_rustfs::AbsolutePathBuf;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};
use cryfs_utils::{data::Data, testutils::data_fixture::DataFixture};

pub const LARGE_FILE_SIZE: usize = 24 * 1024;

pub fn large_symlink_target() -> String {
    (0..1_000)
        .map(|i| format!("pathcomponentforsymlink_{i}"))
        .join("/")
}

pub async fn load_dir_blob<'b, B>(
    fsblobstore: &'b FsBlobStore<B>,
    blob_id: &BlobId,
) -> AsyncDropGuard<DirBlob<'b, B>>
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send,
{
    FsBlob::into_dir(fsblobstore.load(blob_id).await.unwrap().unwrap())
        .await
        .unwrap()
}

pub async fn create_empty_dir<'a, 'b, 'c, B>(
    fsblobstore: &'b FsBlobStore<B>,
    parent: &'a mut DirBlob<'c, B>,
    name: &str,
) -> AsyncDropGuard<DirBlob<'b, B>>
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send,
{
    let new_entry = fsblobstore
        .create_dir_blob(&parent.blob_id())
        .await
        .unwrap();
    add_dir_entry(parent, name, new_entry.blob_id());
    new_entry
}

pub fn add_dir_entry<'a, 'c, B>(parent: &'a mut DirBlob<'c, B>, name: &str, blob_id: BlobId)
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send + 'static,
{
    parent
        .add_entry_dir(
            name.to_string().try_into().unwrap(),
            blob_id,
            Mode::zero().add_dir_flag(),
            Uid::from(1000),
            Gid::from(1000),
            SystemTime::now(),
            SystemTime::now(),
        )
        .unwrap();
}

pub async fn create_empty_file<'a, 'b, 'c, B>(
    fsblobstore: &'b FsBlobStore<B>,
    parent: &'a mut DirBlob<'c, B>,
    name: &str,
) -> FileBlob<'b, B>
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send,
{
    let new_entry = fsblobstore
        .create_file_blob(&parent.blob_id())
        .await
        .unwrap();
    add_file_entry(parent, name, new_entry.blob_id());
    new_entry
}

pub fn add_file_entry<'a, 'c, B>(parent: &'a mut DirBlob<'c, B>, name: &str, blob_id: BlobId)
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send + 'static,
{
    parent
        .add_entry_file(
            name.to_string().try_into().unwrap(),
            blob_id,
            Mode::zero().add_file_flag(),
            Uid::from(1000),
            Gid::from(1000),
            SystemTime::now(),
            SystemTime::now(),
        )
        .unwrap();
}

pub async fn create_symlink<'a, 'b, 'c, B>(
    fsblobstore: &'b FsBlobStore<B>,
    parent: &'a mut DirBlob<'c, B>,
    name: &str,
    target: &str,
) -> SymlinkBlob<'b, B>
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send,
{
    let new_entry = fsblobstore
        .create_symlink_blob(&parent.blob_id(), target)
        .await
        .unwrap();
    add_symlink_entry(parent, name, new_entry.blob_id());
    new_entry
}

pub fn add_symlink_entry<'a, 'c, B>(parent: &'a mut DirBlob<'c, B>, name: &str, blob_id: BlobId)
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send + 'static,
{
    parent
        .add_entry_symlink(
            name.to_string().try_into().unwrap(),
            blob_id,
            Uid::from(1000),
            Gid::from(1000),
            SystemTime::now(),
            SystemTime::now(),
        )
        .unwrap();
}

pub async fn create_large_file<'a, 'b, 'c, B>(
    fsblobstore: &'b FsBlobStore<B>,
    parent: &'a mut DirBlob<'c, B>,
    name: &str,
) -> FileBlob<'b, B>
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send,
{
    let mut file = create_empty_file(fsblobstore, parent, name).await;
    file.write(&data(LARGE_FILE_SIZE, 0), 0).await.unwrap();
    assert!(
        file.num_nodes().await.unwrap() > 1_000,
        "If this fails, we need to make the data larger so it uses enough nodes."
    );

    file
}

pub async fn create_large_symlink<'a, 'b, 'c, B>(
    fsblobstore: &'b FsBlobStore<B>,
    parent: &'a mut DirBlob<'c, B>,
    name: &str,
) -> SymlinkBlob<'b, B>
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send,
{
    let target = large_symlink_target();
    let mut symlink = create_symlink(fsblobstore, parent, name, &target).await;
    assert!(
        symlink.num_nodes().await.unwrap() > 1_000,
        "If this fails, we need to make the target longer so it uses enough nodes."
    );
    symlink
}

pub async fn create_large_dir<'a, 'b, 'c, B>(
    fsblobstore: &'b FsBlobStore<B>,
    parent: &'a mut DirBlob<'c, B>,
    name: &str,
) -> AsyncDropGuard<DirBlob<'b, B>>
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send,
{
    let mut dir = create_empty_dir(fsblobstore, parent, name).await;
    add_entries_to_make_dir_large(fsblobstore, &mut dir).await;
    dir
}

pub async fn add_entries_to_make_dir_large<B>(
    fsblobstore: &FsBlobStore<B>,
    dir: &mut DirBlob<'_, B>,
) where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send,
{
    for i in 0..125 {
        create_empty_dir(fsblobstore, dir, &format!("dir{i}"))
            .await
            .async_drop()
            .await
            .unwrap();
        create_empty_file(fsblobstore, dir, &format!("file{i}")).await;
        create_symlink(
            fsblobstore,
            dir,
            &format!("symlink{i}"),
            &format!("symlink_target_{i}"),
        )
        .await;
    }
    assert!(
        dir.num_nodes().await.unwrap() > 1_000,
        "If this fails, we need to create even more entries to make the directory large enough."
    );
}

#[async_recursion]
pub async fn create_large_dir_with_large_entries<'a, 'b, 'c, B>(
    fsblobstore: &'b FsBlobStore<B>,
    parent: &'a mut DirBlob<'c, B>,
    name: &str,
    levels: usize,
) -> AsyncDropGuard<DirBlob<'b, B>>
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send + Sync,
{
    let mut dir = create_large_dir(fsblobstore, parent, name).await;

    create_large_file(fsblobstore, &mut dir, "large_file").await;
    create_large_symlink(fsblobstore, &mut dir, "large_symlink").await;
    if levels == 0 {
        create_large_dir(fsblobstore, &mut dir, "large_dir")
            .await
            .async_drop()
            .await
            .unwrap();
    } else {
        create_large_dir_with_large_entries(fsblobstore, &mut dir, "large_dir", levels - 1)
            .await
            .async_drop()
            .await
            .unwrap();
    }

    dir
}

#[derive(Debug)]
pub struct SomeBlobs {
    pub root: BlobInfo,
    pub dir1: BlobInfo,
    pub dir2: BlobInfo,
    pub dir1_dir3: BlobInfo,
    pub dir1_dir4: BlobInfo,
    pub dir1_dir3_dir5: BlobInfo,
    pub dir2_dir6: BlobInfo,
    pub dir2_dir7: BlobInfo,
    pub dir2_large_file_1: BlobInfo,
    pub dir2_dir7_large_file_1: BlobInfo,
    pub large_file_1: BlobInfo,
    pub large_file_2: BlobInfo,
    pub large_dir_1: BlobInfo,
    pub large_dir_2: BlobInfo,
    pub dir2_large_symlink_1: BlobInfo,
    pub dir2_dir7_large_symlink_1: BlobInfo,
    pub large_symlink_1: BlobInfo,
    pub large_symlink_2: BlobInfo,
    pub empty_file: BlobInfo,
    pub empty_dir: BlobInfo,
    pub empty_symlink: BlobInfo,
}

pub async fn create_some_blobs<'a, 'b, 'c, B>(
    fsblobstore: &'b FsBlobStore<B>,
    root: &'a mut DirBlob<'c, B>,
) -> SomeBlobs
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send + Sync,
{
    // TODO there's a lot of repetition of path here. We should probably come up with a different API where `create_empty_dir` etc take a `parent: BlobInfo` as argument and return a created `BlobInfo`
    let root_info = blob_info(root.blob_id(), BlobType::Dir, "/");
    let mut dir1 = create_empty_dir(fsblobstore, root, "somedir1").await;
    let dir1_info = blob_info(dir1.blob_id(), BlobType::Dir, "/somedir");
    let mut dir2 = create_empty_dir(fsblobstore, root, "somedir2").await;
    let dir2_info = blob_info(dir2.blob_id(), BlobType::Dir, "/somedir2");
    let mut dir1_dir3 = create_empty_dir(fsblobstore, &mut dir1, "somedir3").await;
    let dir1_dir3_info = blob_info(dir1_dir3.blob_id(), BlobType::Dir, "/somedir1/somedir3");
    let mut dir1_dir4 = create_empty_dir(fsblobstore, &mut dir1, "somedir4").await;
    let dir1_dir4_info = blob_info(dir1_dir4.blob_id(), BlobType::Dir, "/somedir1/somedir4");
    let mut dir1_dir3_dir5 = create_empty_dir(fsblobstore, &mut dir1_dir3, "somedir5").await;
    let dir1_dir3_dir5_info = blob_info(
        dir1_dir3_dir5.blob_id(),
        BlobType::Dir,
        "/somedir1/somedir3/somedir5",
    );
    let mut dir2_dir6 = create_empty_dir(fsblobstore, &mut dir2, "somedir6").await;
    let dir2_dir6_info = blob_info(dir2_dir6.blob_id(), BlobType::Dir, "/somedir2/somedir6");
    let mut dir2_dir7 = create_empty_dir(fsblobstore, &mut dir2, "somedir7").await;
    let dir2_dir7_info = blob_info(dir2_dir7.blob_id(), BlobType::Dir, "/somedir2/somedir7");

    // Let's create a directory, symlink and file with lots of entries (so it'll use multiple nodes)
    let mut large_dir_1 =
        create_large_dir_with_large_entries(fsblobstore, &mut dir2_dir6, "some_large_dir_1", 2)
            .await;
    let large_dir_1_info = blob_info(
        large_dir_1.blob_id(),
        BlobType::Dir,
        "/somedir2/somedir6/some_large_dir_1",
    );
    let mut large_dir_2 =
        create_large_dir_with_large_entries(fsblobstore, &mut dir1_dir4, "some_large_dir_2", 2)
            .await;
    let large_dir_2_info = blob_info(
        large_dir_2.blob_id(),
        BlobType::Dir,
        "/somedir1/somedir4/some_large_dir_2",
    );
    let dir2_dir7_large_symlink_1 =
        create_large_symlink(fsblobstore, &mut dir2_dir7, "some_large_symlink_1").await;
    let dir2_dir7_large_symlink_1_info = blob_info(
        dir2_dir7_large_symlink_1.blob_id(),
        BlobType::Symlink,
        "/somedir2/somedir7/some_large_symlink_1",
    );
    let dir2_large_symlink_1 =
        create_large_symlink(fsblobstore, &mut dir2, "some_large_symlink_2").await;
    let dir2_large_symlink_1_info = blob_info(
        dir2_large_symlink_1.blob_id(),
        BlobType::Symlink,
        "/somedir2/some_large_symlink_2",
    );
    let dir2_dir7_large_file_1 =
        create_large_file(fsblobstore, &mut dir2_dir7, "some_large_file_1").await;
    let dir2_dir7_large_file_1_info = blob_info(
        dir2_dir7_large_file_1.blob_id(),
        BlobType::File,
        "/somedir2/somedir7/some_large_file_1",
    );
    let dir2_large_file_1 = create_large_file(fsblobstore, &mut dir2, "some_large_file_2").await;
    let dir2_large_file_1_info = blob_info(
        dir2_large_file_1.blob_id(),
        BlobType::File,
        "/somedir2/some_large_file_2",
    );

    let empty_file = create_empty_file(fsblobstore, &mut dir1_dir3_dir5, "some_empty_file").await;
    let empty_file_info = blob_info(
        empty_file.blob_id(),
        BlobType::File,
        "/somedir1/somedir3/somedir5/some_empty_file",
    );
    let mut empty_dir = create_empty_dir(fsblobstore, &mut dir2_dir7, "some_empty_dir").await;
    let empty_dir_info = blob_info(
        empty_dir.blob_id(),
        BlobType::Dir,
        "/somedir2/somedir7/some_empty_dir",
    );
    let empty_symlink = create_symlink(fsblobstore, &mut dir1_dir3, "some_empty_symlink", "").await;
    let empty_symlink_info = blob_info(
        empty_symlink.blob_id(),
        BlobType::Symlink,
        "/somedir1/somedir3/some_empty_symlink",
    );

    let result = SomeBlobs {
        root: root_info,
        dir1: dir1_info,
        dir2: dir2_info,
        dir1_dir3: dir1_dir3_info,
        dir1_dir4: dir1_dir4_info,
        dir1_dir3_dir5: dir1_dir3_dir5_info,
        dir2_dir6: dir2_dir6_info,
        dir2_dir7: dir2_dir7_info,
        dir2_dir7_large_file_1: dir2_dir7_large_file_1_info.clone(),
        dir2_large_file_1: dir2_large_file_1_info.clone(),
        large_file_1: dir2_dir7_large_file_1_info,
        large_file_2: dir2_large_file_1_info,
        large_dir_1: large_dir_1_info,
        large_dir_2: large_dir_2_info,
        dir2_dir7_large_symlink_1: dir2_dir7_large_symlink_1_info.clone(),
        large_symlink_1: dir2_dir7_large_symlink_1_info,
        dir2_large_symlink_1: dir2_large_symlink_1_info.clone(),
        large_symlink_2: dir2_large_symlink_1_info,
        empty_file: empty_file_info,
        empty_dir: empty_dir_info,
        empty_symlink: empty_symlink_info,
    };

    large_dir_1.async_drop().await.unwrap();
    large_dir_2.async_drop().await.unwrap();
    dir2_dir7.async_drop().await.unwrap();
    dir2_dir6.async_drop().await.unwrap();
    dir1_dir3_dir5.async_drop().await.unwrap();
    dir1_dir4.async_drop().await.unwrap();
    dir1_dir3.async_drop().await.unwrap();
    dir2.async_drop().await.unwrap();
    dir1.async_drop().await.unwrap();
    empty_dir.async_drop().await.unwrap();

    result
}

fn blob_info(blob_id: BlobId, blob_type: BlobType, path: &str) -> BlobInfo {
    BlobInfo {
        blob_id,
        blob_type,
        path: AbsolutePathBuf::try_from_string(path.to_owned()).unwrap(),
    }
}

pub fn data(size: usize, seed: u64) -> Data {
    DataFixture::new(seed).get(size).into()
}

pub async fn find_an_inner_node_of_a_large_blob<B>(
    nodestore: &DataNodeStore<B>,
    blob_id: &BlobId,
) -> DataInnerNode<B>
where
    B: BlockStore + Send + Sync,
{
    find_inner_node_with_distance_from_root(nodestore, *blob_id.to_root_block_id()).await
}

pub async fn find_inner_node_with_distance_from_root<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
) -> DataInnerNode<B>
where
    B: BlockStore + Send + Sync,
{
    let root_node = nodestore
        .load(root)
        .await
        .unwrap()
        .unwrap()
        .into_inner_node()
        .expect("test blob too small to have more than one node. We need to change the test and increase its size");

    let child_of_root_id = root_node.children().skip(1).next().expect("test blob too small to have more than one child of root. We need to change the test and increase its size");
    let child_of_root = nodestore.load(child_of_root_id).await.unwrap().unwrap().into_inner_node().expect(
        "test blob too small to have more than two levels. We need to change the test and increase its size"
    );
    let child_of_child_of_root_id = child_of_root.children().next().expect("test blob too small to have more than one child of child of root. We need to change the test and increase its size");
    let child_of_child_of_root = nodestore
        .load(child_of_child_of_root_id)
        .await
        .unwrap()
        .unwrap()
        .into_inner_node()
        .expect("test blob too small to have more than three levels. We need to change the test and increase its size");
    child_of_child_of_root
}

pub async fn find_an_inner_node_of_a_small_blob<B>(
    nodestore: &DataNodeStore<B>,
    blob_id: &BlobId,
) -> DataInnerNode<B>
where
    B: BlockStore + Send + Sync,
{
    find_inner_node_without_distance_from_root(nodestore, *blob_id.to_root_block_id()).await
}

pub async fn find_inner_node_without_distance_from_root<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
) -> DataInnerNode<B>
where
    B: BlockStore + Send + Sync,
{
    let root_node = nodestore
        .load(root)
        .await
        .unwrap()
        .unwrap()
        .into_inner_node()
        .expect("test blob too small to have more than one node. We need to change the test and increase its size");

    let child_of_root_id = root_node.children().skip(1).next().expect("test blob too small to have more than one child of root. We need to change the test and increase its size");
    let child_of_root = nodestore.load(child_of_root_id).await.unwrap().unwrap().into_inner_node().expect(
        "test blob too small to have more than two levels. We need to change the test and increase its size"
    );
    child_of_root
}

pub async fn find_leaf_node_of_blob<B>(
    nodestore: &DataNodeStore<B>,
    blob_id: &BlobId,
) -> DataLeafNode<B>
where
    B: BlockStore + Send + Sync,
{
    let mut rng = SmallRng::seed_from_u64(0);
    find_leaf_node(nodestore, *blob_id.to_root_block_id(), &mut rng).await
}

pub async fn find_leaf_id<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
    rng: &mut SmallRng,
) -> BlockId
where
    B: BlockStore + Send + Sync,
{
    *find_leaf_node_and_parent(nodestore, root, rng)
        .await
        .0
        .block_id()
}

pub async fn find_leaf_node<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
    rng: &mut SmallRng,
) -> DataLeafNode<B>
where
    B: BlockStore + Send + Sync,
{
    find_leaf_node_and_parent(nodestore, root, rng).await.0
}

pub async fn find_leaf_id_and_parent<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
    rng: &mut SmallRng,
) -> (BlockId, DataInnerNode<B>, usize)
where
    B: BlockStore + Send + Sync,
{
    let (leaf, parent, index) = find_leaf_node_and_parent(nodestore, root, rng).await;
    (*leaf.block_id(), parent, index)
}

pub async fn find_leaf_node_and_parent<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
    rng: &mut SmallRng,
) -> (DataLeafNode<B>, DataInnerNode<B>, usize)
where
    B: BlockStore + Send + Sync,
{
    let blob_root_node = nodestore
        .load(root)
        .await
        .unwrap()
        .unwrap()
        .into_inner_node()
        .expect("test blob too small to have more than one node. We need to change the test and increase its size");

    _find_leaf_node_and_parent(nodestore, blob_root_node, rng).await
}

#[async_recursion]
pub async fn _find_leaf_node_and_parent<B>(
    nodestore: &DataNodeStore<B>,
    root: DataInnerNode<B>,
    rng: &mut SmallRng,
) -> (DataLeafNode<B>, DataInnerNode<B>, usize)
where
    B: BlockStore + Send + Sync,
{
    let children = root.children();
    let (index, child) = children
        .enumerate()
        .choose(rng)
        .expect("Inner node has no children");
    let child = nodestore.load(child).await.unwrap().unwrap();
    match child {
        DataNode::Inner(inner) => _find_leaf_node_and_parent(nodestore, inner, rng).await,
        DataNode::Leaf(leaf) => (leaf, root, index),
    }
}

pub async fn find_inner_node_of_blob<B>(
    nodestore: &DataNodeStore<B>,
    blob_id: &BlobId,
    depth: u8,
) -> DataInnerNode<B>
where
    B: BlockStore + Send + Sync,
{
    let mut rng = SmallRng::seed_from_u64(0);
    find_inner_node(nodestore, *blob_id.to_root_block_id(), depth, &mut rng).await
}

pub async fn find_inner_node_id<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
    depth: u8,
    rng: &mut SmallRng,
) -> BlockId
where
    B: BlockStore + Send + Sync,
{
    *find_inner_node_and_parent(nodestore, root, depth, rng)
        .await
        .0
        .block_id()
}

pub async fn find_inner_node<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
    depth: u8,
    rng: &mut SmallRng,
) -> DataInnerNode<B>
where
    B: BlockStore + Send + Sync,
{
    find_inner_node_and_parent(nodestore, root, depth, rng)
        .await
        .0
}

pub async fn find_inner_node_id_and_parent<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
    depth: u8,
    rng: &mut SmallRng,
) -> (BlockId, DataInnerNode<B>, usize)
where
    B: BlockStore + Send + Sync,
{
    let (node, parent, index) = find_inner_node_and_parent(nodestore, root, depth, rng).await;
    (*node.block_id(), parent, index)
}

pub async fn find_inner_node_and_parent<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
    depth: u8,
    rng: &mut SmallRng,
) -> (DataInnerNode<B>, DataInnerNode<B>, usize)
where
    B: BlockStore + Send + Sync,
{
    let blob_root_node = nodestore
        .load(root)
        .await
        .unwrap()
        .unwrap()
        .into_inner_node()
        .expect("test blob too small to have more than one node. We need to change the test and increase its size");

    _find_inner_node_and_parent(nodestore, blob_root_node, depth, rng).await
}

#[async_recursion]
pub async fn _find_inner_node_and_parent<B>(
    nodestore: &DataNodeStore<B>,
    root: DataInnerNode<B>,
    depth: u8,
    rng: &mut SmallRng,
) -> (DataInnerNode<B>, DataInnerNode<B>, usize)
where
    B: BlockStore + Send + Sync,
{
    assert!(depth >= 1);

    let children = root.children();
    let (index, child) = children
        .enumerate()
        .choose(rng)
        .expect("Inner node has no children");
    let child = nodestore.load(child).await.unwrap().unwrap();
    let child = child
        .into_inner_node()
        .expect("Tried to find an inner node but found a leaf node");
    if depth == 1 {
        (child, root, index)
    } else {
        _find_inner_node_and_parent(nodestore, child, depth - 1, rng).await
    }
}

pub fn get_descendants_of_dir_blob<'a, 'r, B>(
    fsblobstore: &'a FsBlobStore<B>,
    dir_blob_id: BlobId,
) -> BoxStream<'r, BlobId>
where
    'a: 'r,
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send + Sync,
{
    Box::pin(
        async move {
            let blob = fsblobstore.load(&dir_blob_id).await.unwrap().unwrap();
            let mut blob = FsBlob::into_dir(blob).await.unwrap();
            let children = blob
                .entries()
                .map(|entry| *entry.blob_id())
                .collect::<Vec<_>>();
            let dir_children = blob
                .entries()
                .filter(|entry| entry.mode().has_dir_flag())
                .map(|entry| *entry.blob_id())
                .collect::<Vec<_>>();
            blob.async_drop().await.unwrap();
            let recursive_streams = dir_children
                .into_iter()
                .map(|child_id| get_descendants_if_dir_blob(fsblobstore, child_id))
                .collect::<Vec<_>>();
            let recursive_children = stream::select_all(recursive_streams);
            stream::iter(children.into_iter())
                .chain(recursive_children)
                .boxed()
        }
        .flatten_stream(),
    )
}

pub fn get_descendants_if_dir_blob<'a, 'r, B>(
    fsblobstore: &'a FsBlobStore<B>,
    maybe_dir_blob_id: BlobId,
) -> BoxStream<'r, BlobId>
where
    'a: 'r,
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send + Sync,
{
    Box::pin(
        async move {
            let blob = fsblobstore.load(&maybe_dir_blob_id).await.unwrap().unwrap();
            if let Ok(mut blob) = FsBlob::into_dir(blob).await {
                let children = blob
                    .entries()
                    .map(|entry| *entry.blob_id())
                    .collect::<Vec<_>>();
                let dir_children = blob
                    .entries()
                    .filter(|entry| entry.mode().has_dir_flag())
                    .map(|entry| *entry.blob_id())
                    .collect::<Vec<_>>();
                blob.async_drop().await.unwrap();
                let recursive_streams = dir_children
                    .into_iter()
                    .map(|child_id| get_descendants_if_dir_blob(fsblobstore, child_id))
                    .collect::<Vec<_>>();
                let recursive_children = stream::select_all(recursive_streams);
                stream::iter(children.into_iter())
                    .chain(recursive_children)
                    .boxed()
            } else {
                stream::empty().boxed()
            }
        }
        .flatten_stream(),
    )
}

pub async fn remove_subtree<B>(nodestore: &DataNodeStore<B>, root: BlockId)
where
    B: BlockStore + Send + Sync,
{
    let node = nodestore.load(root).await.unwrap().unwrap();
    DataTree::remove_subtree(nodestore, node).await.unwrap();
}
