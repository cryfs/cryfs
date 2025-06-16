use anyhow::Result;
use async_trait::async_trait;
use futures::{
    future::FutureExt,
    stream::{self, BoxStream, StreamExt},
};
use itertools::Itertools;
use rand::{SeedableRng, rngs::SmallRng, seq::IteratorRandom};
use std::fmt::Debug;
use std::time::SystemTime;

use crate::FilesystemFixture;
use cryfs_blobstore::{
    BlobId, BlobStore, DataInnerNode, DataLeafNode, DataNode, DataNodeStore, DataTree,
};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_check::{
    BlobReference, BlobReferenceWithId, CorruptedError, NodeInfoAsSeenByLookingAtNode,
    NodeUnreferencedError,
};
use cryfs_filesystem::filesystem::fsblobstore::BlobType;
use cryfs_filesystem::{
    filesystem::fsblobstore::{DirBlob, FileBlob, FsBlob, FsBlobStore, SymlinkBlob},
    utils::fs_types::{Gid, Mode, Uid},
};
use cryfs_rustfs::{AbsolutePathBuf, FsError};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    with_async_drop_2,
};
use cryfs_utils::{data::Data, testutils::data_fixture::DataFixture};

pub const LARGE_FILE_SIZE: usize = 24 * 1024;

#[derive(Debug)]
pub struct CreatedDirBlob<'a, B>
where
    B: BlobStore + Debug + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    blob: AsyncDropGuard<FsBlob<'a, B>>,
    path: AbsolutePathBuf,
}

impl<'a, B> CreatedDirBlob<'a, B>
where
    B: BlobStore + Debug + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    pub fn new(blob: AsyncDropGuard<FsBlob<'a, B>>, path: AbsolutePathBuf) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { blob, path })
    }

    pub fn blob(&mut self) -> &mut FsBlob<'a, B> {
        &mut self.blob
    }

    pub fn dir_blob(&self) -> &DirBlob<'a, B> {
        self.blob
            .as_dir()
            .expect("We just created this dir blob and now it is a different type")
    }

    pub fn dir_blob_mut(&mut self) -> &mut DirBlob<'a, B> {
        self.blob
            .as_dir_mut()
            .expect("We just created this dir blob and now it is a different type")
    }

    pub fn into_blob(this: AsyncDropGuard<Self>) -> AsyncDropGuard<FsBlob<'a, B>> {
        this.unsafe_into_inner_dont_drop().blob
    }
}

#[async_trait]
impl<'a, B> AsyncDrop for CreatedDirBlob<'a, B>
where
    B: BlobStore + Debug + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        self.blob.async_drop().await
    }
}

impl<'a, B> From<&CreatedDirBlob<'a, B>> for BlobReferenceWithId
where
    B: BlobStore + Debug + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    fn from(blob: &CreatedDirBlob<'a, B>) -> Self {
        Self {
            blob_id: blob.blob.blob_id(),
            referenced_as: BlobReference {
                blob_type: BlobType::Dir,
                parent_id: blob.blob.parent(),
                path: blob.path.clone(),
            },
        }
    }
}

#[derive(Debug)]
pub struct CreatedFileBlob<'a, B>
where
    B: BlobStore + Debug + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    blob: AsyncDropGuard<FsBlob<'a, B>>,
    path: AbsolutePathBuf,
}

impl<'a, B> CreatedFileBlob<'a, B>
where
    B: BlobStore + Debug + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    pub fn new(blob: AsyncDropGuard<FsBlob<'a, B>>, path: AbsolutePathBuf) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { blob, path })
    }

    pub fn file_blob(&self) -> &FileBlob<'a, B> {
        self.blob
            .as_file()
            .expect("We just created this file blob and now it is a different type")
    }

    pub fn file_blob_mut(&mut self) -> &mut FileBlob<'a, B> {
        self.blob
            .as_file_mut()
            .expect("We just created this file blob and now it is a different type")
    }
}

impl<'a, B> From<&CreatedFileBlob<'a, B>> for BlobReferenceWithId
where
    B: BlobStore + Debug + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    fn from(blob: &CreatedFileBlob<'a, B>) -> Self {
        Self {
            blob_id: blob.blob.blob_id(),
            referenced_as: BlobReference {
                blob_type: BlobType::File,
                parent_id: blob.blob.parent(),
                path: blob.path.clone(),
            },
        }
    }
}

#[async_trait]
impl<'a, B> AsyncDrop for CreatedFileBlob<'a, B>
where
    B: BlobStore + Debug + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        self.blob.async_drop().await
    }
}

#[derive(Debug)]
pub struct CreatedSymlinkBlob<'a, B>
where
    B: BlobStore + Debug + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    blob: AsyncDropGuard<FsBlob<'a, B>>,
    path: AbsolutePathBuf,
}

impl<'a, B> CreatedSymlinkBlob<'a, B>
where
    B: BlobStore + Debug + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    pub fn new(blob: AsyncDropGuard<FsBlob<'a, B>>, path: AbsolutePathBuf) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { blob, path })
    }

    pub fn symlink_blob(&self) -> &SymlinkBlob<'a, B> {
        self.blob
            .as_symlink()
            .expect("We just created this symlink blob and now it is a different type")
    }

    pub fn symlink_blob_mut(&mut self) -> &mut SymlinkBlob<'a, B> {
        self.blob
            .as_symlink_mut()
            .expect("We just created this symlink blob and now it is a different type")
    }
}

#[async_trait]
impl<'a, B> AsyncDrop for CreatedSymlinkBlob<'a, B>
where
    B: BlobStore + Debug + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        self.blob.async_drop().await
    }
}

impl<'a, B> From<&CreatedSymlinkBlob<'a, B>> for BlobReferenceWithId
where
    B: BlobStore + Debug + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    fn from(blob: &CreatedSymlinkBlob<'a, B>) -> Self {
        Self {
            blob_id: blob.blob.blob_id(),
            referenced_as: BlobReference {
                blob_type: BlobType::Symlink,
                parent_id: blob.blob.parent(),
                path: blob.path.clone(),
            },
        }
    }
}

pub fn large_symlink_target() -> String {
    (0..1_000)
        .map(|i| format!("pathcomponentforsymlink_{i}"))
        .join("/")
}

pub async fn load_blob<'b, B>(
    fsblobstore: &'b FsBlobStore<B>,
    blob_id: &BlobId,
) -> AsyncDropGuard<FsBlob<'b, B>>
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send,
{
    fsblobstore.load(blob_id).await.unwrap().unwrap()
}

pub async fn create_empty_dir<'a, 'b, 'c, B>(
    fsblobstore: &'b FsBlobStore<B>,
    parent: &'a mut CreatedDirBlob<'c, B>,
    name: &str,
) -> AsyncDropGuard<CreatedDirBlob<'b, B>>
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send,
{
    let mut parent_dir = parent.blob.as_dir_mut().unwrap();
    let new_entry = fsblobstore
        .create_dir_blob(&parent_dir.blob_id())
        .await
        .unwrap();
    add_dir_entry(&mut parent_dir, name, new_entry.blob_id());
    CreatedDirBlob::new(new_entry, parent.path.join(name.try_into().unwrap()))
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
    parent: &'a mut CreatedDirBlob<'c, B>,
    name: &str,
) -> AsyncDropGuard<CreatedFileBlob<'b, B>>
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send,
{
    let mut parent_dir = parent.blob.as_dir_mut().unwrap();
    let new_entry = fsblobstore
        .create_file_blob(&parent_dir.blob_id())
        .await
        .unwrap();
    add_file_entry(&mut parent_dir, name, new_entry.blob_id());
    CreatedFileBlob::new(new_entry, parent.path.join(name.try_into().unwrap()))
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
    parent: &'a mut CreatedDirBlob<'c, B>,
    name: &str,
    target: &str,
) -> AsyncDropGuard<CreatedSymlinkBlob<'b, B>>
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send,
{
    let mut parent_dir = parent.blob.as_dir_mut().unwrap();
    let new_entry = fsblobstore
        .create_symlink_blob(&parent_dir.blob_id(), target)
        .await
        .unwrap();
    add_symlink_entry(&mut parent_dir, name, new_entry.blob_id());
    CreatedSymlinkBlob::new(new_entry, parent.path.join(name.try_into().unwrap()))
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
    parent: &'a mut CreatedDirBlob<'c, B>,
    name: &str,
) -> AsyncDropGuard<CreatedFileBlob<'b, B>>
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send,
{
    let mut blob = create_empty_file(fsblobstore, parent, name).await;
    let file = blob.file_blob_mut();

    file.write(&data(LARGE_FILE_SIZE, 0), 0).await.unwrap();
    assert!(
        file.num_nodes().await.unwrap() > 1_000,
        "If this fails, we need to make the data larger so it uses enough nodes."
    );

    blob
}

pub async fn create_large_symlink<'a, 'b, 'c, B>(
    fsblobstore: &'b FsBlobStore<B>,
    parent: &'a mut CreatedDirBlob<'c, B>,
    name: &str,
) -> AsyncDropGuard<CreatedSymlinkBlob<'b, B>>
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send,
{
    let target = large_symlink_target();
    let mut blob = create_symlink(fsblobstore, parent, name, &target).await;
    let symlink = blob.symlink_blob_mut();
    assert!(
        symlink.num_nodes().await.unwrap() > 1_000,
        "If this fails, we need to make the target longer so it uses enough nodes."
    );
    blob
}

pub async fn create_large_dir<'a, 'b, 'c, B>(
    fsblobstore: &'b FsBlobStore<B>,
    parent: &'a mut CreatedDirBlob<'c, B>,
    name: &str,
) -> AsyncDropGuard<CreatedDirBlob<'b, B>>
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send,
{
    let mut dir = create_empty_dir(fsblobstore, parent, name).await;
    add_entries_to_make_dir_large(fsblobstore, &mut dir).await;
    dir
}

pub async fn add_entries_to_make_dir_large<B>(
    fsblobstore: &FsBlobStore<B>,
    dir: &mut CreatedDirBlob<'_, B>,
) where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send,
{
    for i in 0..125 {
        create_empty_dir(fsblobstore, dir, &format!("dir{i}"))
            .await
            .async_drop()
            .await
            .unwrap();
        create_empty_file(fsblobstore, dir, &format!("file{i}"))
            .await
            .async_drop()
            .await
            .unwrap();
        create_symlink(
            fsblobstore,
            dir,
            &format!("symlink{i}"),
            &format!("symlink_target_{i}"),
        )
        .await
        .async_drop()
        .await
        .unwrap();
    }
    assert!(
        dir.dir_blob_mut().num_nodes().await.unwrap() > 1_000,
        "If this fails, we need to create even more entries to make the directory large enough."
    );
}

pub async fn create_large_dir_with_large_entries<'a, 'b, 'c, B>(
    fsblobstore: &'b FsBlobStore<B>,
    parent: &'a mut CreatedDirBlob<'c, B>,
    name: &str,
    levels: usize,
) -> AsyncDropGuard<CreatedDirBlob<'b, B>>
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send + Sync,
{
    let mut dir = create_large_dir(fsblobstore, parent, name).await;

    create_large_file(fsblobstore, &mut dir, "large_file")
        .await
        .async_drop()
        .await
        .unwrap();
    create_large_symlink(fsblobstore, &mut dir, "large_symlink")
        .await
        .async_drop()
        .await
        .unwrap();
    if levels == 0 {
        create_large_dir(fsblobstore, &mut dir, "large_dir")
            .await
            .async_drop()
            .await
            .unwrap();
    } else {
        Box::pin(create_large_dir_with_large_entries(
            fsblobstore,
            &mut dir,
            "large_dir",
            levels - 1,
        ))
        .await
        .async_drop()
        .await
        .unwrap();
    }

    dir
}

#[derive(Debug)]
pub struct SomeBlobs {
    pub root: BlobReferenceWithId,
    pub dir1: BlobReferenceWithId,
    pub dir2: BlobReferenceWithId,
    pub dir1_dir3: BlobReferenceWithId,
    pub dir1_dir4: BlobReferenceWithId,
    pub dir1_dir3_dir5: BlobReferenceWithId,
    pub dir2_dir6: BlobReferenceWithId,
    pub dir2_dir7: BlobReferenceWithId,
    pub dir2_large_file_1: BlobReferenceWithId,
    pub dir2_dir7_large_file_1: BlobReferenceWithId,
    pub large_file_1: BlobReferenceWithId,
    pub large_file_2: BlobReferenceWithId,
    pub large_dir_1: BlobReferenceWithId,
    pub large_dir_2: BlobReferenceWithId,
    pub dir2_large_symlink_1: BlobReferenceWithId,
    pub dir2_dir7_large_symlink_1: BlobReferenceWithId,
    pub large_symlink_1: BlobReferenceWithId,
    pub large_symlink_2: BlobReferenceWithId,
    pub empty_file: BlobReferenceWithId,
    pub empty_dir: BlobReferenceWithId,
    pub empty_symlink: BlobReferenceWithId,
}

pub async fn create_some_blobs<'a, 'b, 'c, B>(
    fsblobstore: &'b FsBlobStore<B>,
    root: &'a mut CreatedDirBlob<'c, B>,
) -> SomeBlobs
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send + Sync,
{
    let mut dir1 = create_empty_dir(fsblobstore, root, "somedir1").await;
    let mut dir2 = create_empty_dir(fsblobstore, root, "somedir2").await;
    let mut dir1_dir3 = create_empty_dir(fsblobstore, &mut dir1, "somedir3").await;
    let mut dir1_dir4 = create_empty_dir(fsblobstore, &mut dir1, "somedir4").await;
    let mut dir1_dir3_dir5 = create_empty_dir(fsblobstore, &mut dir1_dir3, "somedir5").await;
    let mut dir2_dir6 = create_empty_dir(fsblobstore, &mut dir2, "somedir6").await;
    let mut dir2_dir7 = create_empty_dir(fsblobstore, &mut dir2, "somedir7").await;

    // Let's create a directory, symlink and file with lots of entries (so it'll use multiple nodes)
    let mut large_dir_1 =
        create_large_dir_with_large_entries(fsblobstore, &mut dir2_dir6, "some_large_dir_1", 2)
            .await;
    let mut large_dir_2 =
        create_large_dir_with_large_entries(fsblobstore, &mut dir1_dir4, "some_large_dir_2", 2)
            .await;
    let mut dir2_dir7_large_symlink_1 =
        create_large_symlink(fsblobstore, &mut dir2_dir7, "some_large_symlink_1").await;
    let mut dir2_large_symlink_1 =
        create_large_symlink(fsblobstore, &mut dir2, "some_large_symlink_2").await;
    let mut dir2_dir7_large_file_1 =
        create_large_file(fsblobstore, &mut dir2_dir7, "some_large_file_1").await;
    let mut dir2_large_file_1 =
        create_large_file(fsblobstore, &mut dir2, "some_large_file_2").await;

    let mut empty_file =
        create_empty_file(fsblobstore, &mut dir1_dir3_dir5, "some_empty_file").await;
    let mut empty_dir = create_empty_dir(fsblobstore, &mut dir2_dir7, "some_empty_dir").await;
    let mut empty_symlink =
        create_symlink(fsblobstore, &mut dir1_dir3, "some_empty_symlink", "").await;

    let result = SomeBlobs {
        root: (&*root).into(),
        dir1: (&*dir1).into(),
        dir2: (&*dir2).into(),
        dir1_dir3: (&*dir1_dir3).into(),
        dir1_dir4: (&*dir1_dir4).into(),
        dir1_dir3_dir5: (&*dir1_dir3_dir5).into(),
        dir2_dir6: (&*dir2_dir6).into(),
        dir2_dir7: (&*dir2_dir7).into(),
        dir2_dir7_large_file_1: (&*dir2_dir7_large_file_1).into(),
        dir2_large_file_1: (&*dir2_large_file_1).into(),
        large_file_1: (&*dir2_dir7_large_file_1).into(),
        large_file_2: (&*dir2_large_file_1).into(),
        large_dir_1: (&*large_dir_1).into(),
        large_dir_2: (&*large_dir_2).into(),
        dir2_dir7_large_symlink_1: (&*dir2_dir7_large_symlink_1).into(),
        large_symlink_1: (&*dir2_dir7_large_symlink_1).into(),
        dir2_large_symlink_1: (&*dir2_large_symlink_1).into(),
        large_symlink_2: (&*dir2_large_symlink_1).into(),
        empty_file: (&*empty_file).into(),
        empty_dir: (&*empty_dir).into(),
        empty_symlink: (&*empty_symlink).into(),
    };

    large_dir_1.async_drop().await.unwrap();
    large_dir_2.async_drop().await.unwrap();
    dir2_dir7.async_drop().await.unwrap();
    dir2_dir6.async_drop().await.unwrap();
    dir2_dir7_large_symlink_1.async_drop().await.unwrap();
    dir2_large_symlink_1.async_drop().await.unwrap();
    dir1_dir3_dir5.async_drop().await.unwrap();
    dir1_dir4.async_drop().await.unwrap();
    dir1_dir3.async_drop().await.unwrap();
    dir2.async_drop().await.unwrap();
    dir1.async_drop().await.unwrap();
    empty_dir.async_drop().await.unwrap();
    empty_symlink.async_drop().await.unwrap();
    empty_file.async_drop().await.unwrap();
    dir2_dir7_large_file_1.async_drop().await.unwrap();
    dir2_large_file_1.async_drop().await.unwrap();

    result
}

pub fn data(size: usize, seed: u64) -> Data {
    DataFixture::new(seed).get(size).into()
}

pub async fn find_an_inner_node_of_a_large_blob<B>(
    nodestore: &DataNodeStore<B>,
    blob_id: &BlobId,
) -> DataInnerNode<B>
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
{
    find_an_inner_node_of_a_large_blob_with_parent_id(nodestore, blob_id)
        .await
        .0
}

pub async fn find_an_inner_node_of_a_large_blob_with_parent_id<B>(
    nodestore: &DataNodeStore<B>,
    blob_id: &BlobId,
) -> (DataInnerNode<B>, BlockId)
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
{
    find_inner_node_with_distance_from_root_with_parent_id(nodestore, *blob_id.to_root_block_id())
        .await
}

pub async fn find_inner_node_with_distance_from_root<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
) -> DataInnerNode<B>
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
{
    find_inner_node_with_distance_from_root_with_parent_id(nodestore, root)
        .await
        .0
}

pub async fn find_inner_node_with_distance_from_root_with_parent_id<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
) -> (DataInnerNode<B>, BlockId)
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
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
    (child_of_child_of_root, child_of_root_id)
}

pub async fn find_an_inner_node_of_a_small_blob<B>(
    nodestore: &DataNodeStore<B>,
    blob_id: &BlobId,
) -> DataInnerNode<B>
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
{
    find_an_inner_node_of_a_small_blob_with_parent_id(nodestore, blob_id)
        .await
        .0
}

pub async fn find_an_inner_node_of_a_small_blob_with_parent_id<B>(
    nodestore: &DataNodeStore<B>,
    blob_id: &BlobId,
) -> (DataInnerNode<B>, BlockId)
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
{
    find_inner_node_without_distance_from_root_with_parent_id(
        nodestore,
        *blob_id.to_root_block_id(),
    )
    .await
}

pub async fn find_inner_node_without_distance_from_root_with_parent_id<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
) -> (DataInnerNode<B>, BlockId)
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
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
    (child_of_root, root)
}

pub async fn find_leaf_node_of_blob<B>(
    nodestore: &DataNodeStore<B>,
    blob_id: &BlobId,
) -> DataLeafNode<B>
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
{
    find_leaf_node_of_blob_with_parent_id(nodestore, blob_id)
        .await
        .0
}

pub async fn find_leaf_node_of_blob_with_parent_id<B>(
    nodestore: &DataNodeStore<B>,
    blob_id: &BlobId,
) -> (DataLeafNode<B>, BlockId)
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
{
    let mut rng = SmallRng::seed_from_u64(0);
    find_leaf_node_with_parent_id(nodestore, *blob_id.to_root_block_id(), &mut rng).await
}

pub async fn find_leaf_id<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
    rng: &mut SmallRng,
) -> BlockId
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
{
    *find_leaf_node_and_parent(nodestore, root, rng)
        .await
        .0
        .block_id()
}

pub async fn find_leaf_node_with_parent_id<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
    rng: &mut SmallRng,
) -> (DataLeafNode<B>, BlockId)
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
{
    let (leaf, parent, _index) = find_leaf_node_and_parent(nodestore, root, rng).await;
    (leaf, *parent.block_id())
}

pub async fn find_leaf_node<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
    rng: &mut SmallRng,
) -> DataLeafNode<B>
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
{
    find_leaf_node_and_parent(nodestore, root, rng).await.0
}

pub async fn find_leaf_id_and_parent<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
    rng: &mut SmallRng,
) -> (BlockId, DataInnerNode<B>, usize)
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
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
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
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

pub async fn _find_leaf_node_and_parent<B>(
    nodestore: &DataNodeStore<B>,
    root: DataInnerNode<B>,
    rng: &mut SmallRng,
) -> (DataLeafNode<B>, DataInnerNode<B>, usize)
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
{
    let children = root.children();
    let (index, child) = children
        .enumerate()
        .choose(rng)
        .expect("Inner node has no children");
    let child = nodestore.load(child).await.unwrap().unwrap();
    match child {
        DataNode::Inner(inner) => Box::pin(_find_leaf_node_and_parent(nodestore, inner, rng)).await,
        DataNode::Leaf(leaf) => (leaf, root, index),
    }
}

pub async fn find_inner_node_of_blob<B>(
    nodestore: &DataNodeStore<B>,
    blob_id: &BlobId,
    depth_distance_from_root: u8,
) -> DataInnerNode<B>
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
{
    let mut rng = SmallRng::seed_from_u64(0);
    find_inner_node(
        nodestore,
        *blob_id.to_root_block_id(),
        depth_distance_from_root,
        &mut rng,
    )
    .await
}

pub async fn find_inner_node_id<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
    depth_distance_from_root: u8,
    rng: &mut SmallRng,
) -> BlockId
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
{
    *find_inner_node_and_parent(nodestore, root, depth_distance_from_root, rng)
        .await
        .0
        .block_id()
}

pub async fn find_inner_node<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
    depth_distance_from_root: u8,
    rng: &mut SmallRng,
) -> DataInnerNode<B>
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
{
    find_inner_node_and_parent(nodestore, root, depth_distance_from_root, rng)
        .await
        .0
}

pub async fn find_inner_node_id_and_parent<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
    depth_distance_from_root: u8,
    rng: &mut SmallRng,
) -> (BlockId, DataInnerNode<B>, usize)
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
{
    let (node, parent, index) =
        find_inner_node_and_parent(nodestore, root, depth_distance_from_root, rng).await;
    (*node.block_id(), parent, index)
}

pub async fn find_inner_node_and_parent<B>(
    nodestore: &DataNodeStore<B>,
    root: BlockId,
    depth_distance_from_root: u8,
    rng: &mut SmallRng,
) -> (DataInnerNode<B>, DataInnerNode<B>, usize)
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
{
    let blob_root_node = nodestore
        .load(root)
        .await
        .unwrap()
        .unwrap()
        .into_inner_node()
        .expect("test blob too small to have more than one node. We need to change the test and increase its size");

    _find_inner_node_and_parent(nodestore, blob_root_node, depth_distance_from_root, rng).await
}

pub async fn _find_inner_node_and_parent<B>(
    nodestore: &DataNodeStore<B>,
    root: DataInnerNode<B>,
    depth_distance_from_root: u8,
    rng: &mut SmallRng,
) -> (DataInnerNode<B>, DataInnerNode<B>, usize)
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
{
    assert!(depth_distance_from_root >= 1);

    let children = root.children();
    let (index, child) = children
        .enumerate()
        .choose(rng)
        .expect("Inner node has no children");
    let child = nodestore.load(child).await.unwrap().unwrap();
    let child = child
        .into_inner_node()
        .expect("Tried to find an inner node but found a leaf node");
    if depth_distance_from_root == 1 {
        (child, root, index)
    } else {
        Box::pin(_find_inner_node_and_parent(
            nodestore,
            child,
            depth_distance_from_root - 1,
            rng,
        ))
        .await
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
            let (children, dir_children) = with_async_drop_2!(blob, {
                let blob = blob.as_dir().expect("Expected a directory blob");
                let children = blob
                    .entries()
                    .map(|entry| *entry.blob_id())
                    .collect::<Vec<_>>();
                let dir_children = blob
                    .entries()
                    .filter(|entry| entry.mode().has_dir_flag())
                    .map(|entry| *entry.blob_id())
                    .collect::<Vec<_>>();
                Ok::<_, anyhow::Error>((children, dir_children))
            })
            .unwrap();
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
            with_async_drop_2!(blob, {
                if let Ok(blob) = blob.as_dir() {
                    let children = blob
                        .entries()
                        .map(|entry| *entry.blob_id())
                        .collect::<Vec<_>>();
                    let dir_children = blob
                        .entries()
                        .filter(|entry| entry.mode().has_dir_flag())
                        .map(|entry| *entry.blob_id())
                        .collect::<Vec<_>>();
                    let recursive_streams = dir_children
                        .into_iter()
                        .map(|child_id| get_descendants_if_dir_blob(fsblobstore, child_id))
                        .collect::<Vec<_>>();
                    let recursive_children = stream::select_all(recursive_streams);
                    Ok::<_, anyhow::Error>(
                        stream::iter(children.into_iter())
                            .chain(recursive_children)
                            .boxed(),
                    )
                } else {
                    Ok::<_, anyhow::Error>(stream::empty().boxed())
                }
            })
            .unwrap()
        }
        .flatten_stream(),
    )
}

pub async fn remove_subtree<B>(nodestore: &DataNodeStore<B>, root: BlockId)
where
    B: BlockStore<Block: Send + Sync> + AsyncDrop + Debug + Send + Sync,
{
    let node = nodestore.load(root).await.unwrap().unwrap();
    DataTree::remove_subtree(nodestore, node).await.unwrap();
}

pub fn load_node_info<B>(node: &DataNode<B>) -> NodeInfoAsSeenByLookingAtNode
where
    B: BlockStore + Send + Sync,
{
    match node {
        DataNode::Inner(node) => NodeInfoAsSeenByLookingAtNode::InnerNode {
            depth: node.depth(),
        },
        DataNode::Leaf(_) => NodeInfoAsSeenByLookingAtNode::LeafNode,
    }
}

pub async fn expect_blobs_to_have_unreferenced_root_nodes<I>(
    fs_fixture: &FilesystemFixture,
    blobs: I,
) -> impl Iterator<Item = CorruptedError> + use<I>
where
    I: IntoIterator<Item = BlobId>,
    I::IntoIter: Send + 'static,
{
    expect_nodes_to_be_unreferenced(
        fs_fixture,
        blobs.into_iter().map(|blob_id| *blob_id.to_root_block_id()),
    )
    .await
}

pub async fn expect_nodes_to_be_unreferenced<I>(
    fs_fixture: &FilesystemFixture,
    nodes: I,
) -> impl Iterator<Item = CorruptedError> + use<I>
where
    I: IntoIterator<Item = BlockId>,
    I::IntoIter: Send + 'static,
{
    fs_fixture
        .load_node_infos(nodes.into_iter())
        .await
        .map(|(node_id, node_info)| NodeUnreferencedError { node_id, node_info }.into())
}

pub async fn expect_node_to_be_unreferenced(
    fs_fixture: &FilesystemFixture,
    node_id: BlockId,
) -> CorruptedError {
    let node_info = fs_fixture.load_node_info(node_id).await;
    NodeUnreferencedError { node_id, node_info }.into()
}
