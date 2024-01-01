use async_recursion::async_recursion;
use futures::{
    future::FutureExt,
    stream::{self, BoxStream, StreamExt},
};
use itertools::Itertools;
use std::fmt::Debug;
use std::time::SystemTime;

use cryfs_blobstore::{BlobId, BlobStore, DataInnerNode, DataNodeStore};
use cryfs_blockstore::BlockStore;
use cryfs_cryfs::{
    filesystem::fsblobstore::{DirBlob, FileBlob, FsBlob, FsBlobStore, SymlinkBlob},
    utils::fs_types::{Gid, Mode, Uid},
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};
use cryfs_utils::{data::Data, testutils::data_fixture::DataFixture};

pub const LARGE_FILE_SIZE: usize = 24 * 1024;

fn large_symlink_target() -> String {
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

pub async fn create_dir<'a, 'b, 'c, B>(
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

pub async fn create_file<'a, 'b, 'c, B>(
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
    let mut file = create_file(fsblobstore, parent, name).await;
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
    let mut dir = create_dir(fsblobstore, parent, name).await;

    for i in 0..100 {
        create_dir(fsblobstore, &mut dir, &format!("dir{i}"))
            .await
            .async_drop()
            .await
            .unwrap();
        create_file(fsblobstore, &mut dir, &format!("file{i}")).await;
        create_symlink(
            fsblobstore,
            &mut dir,
            &format!("symlink{i}"),
            &format!("symlink_target_{i}"),
        )
        .await;
    }
    assert!(
        dir.num_nodes().await.unwrap() > 1_000,
        "If this fails, we need to create even more entries to make the directory large enough."
    );

    dir
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
    pub root: BlobId,
    pub dir1: BlobId,
    pub dir2: BlobId,
    pub dir1_dir3: BlobId,
    pub dir1_dir4: BlobId,
    pub dir1_dir3_dir5: BlobId,
    pub dir2_dir6: BlobId,
    pub dir2_dir7: BlobId,
    pub large_file: BlobId,
    pub large_dir: BlobId,
    pub large_symlink: BlobId,
}

pub async fn create_some_blobs<'a, 'b, 'c, B>(
    fsblobstore: &'b FsBlobStore<B>,
    root: &'a mut DirBlob<'c, B>,
) -> SomeBlobs
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send + Sync,
{
    let mut dir1 = create_dir(fsblobstore, root, "dir1").await;
    let mut dir2 = create_dir(fsblobstore, &mut dir1, "dir2").await;
    let mut dir1_dir3 = create_dir(fsblobstore, &mut dir1, "dir3").await;
    let mut dir1_dir4 = create_dir(fsblobstore, &mut dir1, "dir4").await;
    let mut dir1_dir3_dir5 = create_dir(fsblobstore, &mut dir1_dir3, "dir5").await;
    let mut dir2_dir6 = create_dir(fsblobstore, &mut dir2, "dir6").await;
    let mut dir2_dir7 = create_dir(fsblobstore, &mut dir2, "dir7").await;

    // Let's create a directory, symlink and file with lots of entries (so it'll use multiple nodes)
    let mut large_dir =
        create_large_dir_with_large_entries(fsblobstore, &mut dir2_dir6, "large_dir", 2).await;
    let large_symlink = create_large_symlink(fsblobstore, &mut dir2_dir7, "large_symlink").await;
    let large_file = create_large_file(fsblobstore, &mut dir2_dir7, "large_file").await;

    let result = SomeBlobs {
        root: root.blob_id(),
        dir1: dir1.blob_id(),
        dir2: dir2.blob_id(),
        dir1_dir3: dir1_dir3.blob_id(),
        dir1_dir4: dir1_dir4.blob_id(),
        dir1_dir3_dir5: dir1_dir3_dir5.blob_id(),
        dir2_dir6: dir2_dir6.blob_id(),
        dir2_dir7: dir2_dir7.blob_id(),
        large_file: large_file.blob_id(),
        large_dir: large_dir.blob_id(),
        large_symlink: large_symlink.blob_id(),
    };

    large_dir.async_drop().await.unwrap();
    dir2_dir7.async_drop().await.unwrap();
    dir2_dir6.async_drop().await.unwrap();
    dir1_dir3_dir5.async_drop().await.unwrap();
    dir1_dir4.async_drop().await.unwrap();
    dir1_dir3.async_drop().await.unwrap();
    dir2.async_drop().await.unwrap();
    dir1.async_drop().await.unwrap();

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
    B: BlockStore + Send + Sync,
{
    let blob_root_node = nodestore
        .load(*blob_id.to_root_block_id())
        .await
        .unwrap()
        .unwrap()
        .into_inner_node()
        .expect("test blob too small to have more than one node. We need to change the test and increase its size");

    let child_of_root_id = blob_root_node.children().skip(1).next().expect("test blob too small to have more than one child of root. We need to change the test and increase its size");
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
                .map(|child_id| get_descendants_of_dir_blob(fsblobstore, child_id))
                .collect::<Vec<_>>();
            let recursive_children = stream::select_all(recursive_streams);
            stream::iter(children.into_iter()).chain(recursive_children)
        }
        .flatten_stream(),
    )
}
