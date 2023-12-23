use cryfs_utils::{data::Data, testutils::data_fixture::DataFixture};
use itertools::Itertools;
use std::fmt::Debug;
use std::time::SystemTime;

use cryfs_blobstore::BlobStore;
use cryfs_cryfs::{
    filesystem::fsblobstore::{DirBlob, FileBlob, FsBlob, FsBlobStore, SymlinkBlob},
    utils::fs_types::{Gid, Mode, Uid},
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

pub async fn create_dir<'a, 'b, 'c, B>(
    fsblobstore: &'b FsBlobStore<B>,
    parent: &'a mut DirBlob<'c, B>,
    name: &str,
) -> AsyncDropGuard<DirBlob<'b, B>>
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send,
{
    let mut new_entry = fsblobstore
        .create_dir_blob(&parent.blob_id())
        .await
        .unwrap();
    parent
        .add_entry_dir(
            name.to_string().try_into().unwrap(),
            new_entry.blob_id(),
            Mode::zero().add_dir_flag(),
            Uid::from(1000),
            Gid::from(1000),
            SystemTime::now(),
            SystemTime::now(),
        )
        .unwrap();
    new_entry
}

pub async fn create_file<'a, 'b, 'c, B>(
    fsblobstore: &'b FsBlobStore<B>,
    parent: &'a mut DirBlob<'c, B>,
    name: &str,
) -> FileBlob<'b, B>
where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send,
{
    let mut new_entry = fsblobstore
        .create_file_blob(&parent.blob_id())
        .await
        .unwrap();
    parent
        .add_entry_file(
            name.to_string().try_into().unwrap(),
            new_entry.blob_id(),
            Mode::zero().add_file_flag(),
            Uid::from(1000),
            Gid::from(1000),
            SystemTime::now(),
            SystemTime::now(),
        )
        .unwrap();
    new_entry
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
    let mut new_entry = fsblobstore
        .create_symlink_blob(&parent.blob_id(), target)
        .await
        .unwrap();
    parent
        .add_entry_symlink(
            name.to_string().try_into().unwrap(),
            new_entry.blob_id(),
            Uid::from(1000),
            Gid::from(1000),
            SystemTime::now(),
            SystemTime::now(),
        )
        .unwrap();
    new_entry
}

pub async fn create_some_files_directories_and_symlinks<'a, 'b, 'c, B>(
    fsblobstore: &'b FsBlobStore<B>,
    root: &'a mut DirBlob<'c, B>,
) where
    B: BlobStore + Debug + AsyncDrop<Error = anyhow::Error> + Send,
{
    let mut dir1 = create_dir(fsblobstore, root, "dir1").await;
    let mut dir2 = create_dir(fsblobstore, &mut dir1, "dir2").await;
    let mut dir1_dir3 = create_dir(fsblobstore, &mut dir1, "dir3").await;
    let mut dir1_dir4 = create_dir(fsblobstore, &mut dir1, "dir4").await;
    let mut dir1_dir3_dir5 = create_dir(fsblobstore, &mut dir1_dir3, "dir5").await;
    let mut dir2_dir6 = create_dir(fsblobstore, &mut dir2, "dir6").await;
    let mut dir2_dir7 = create_dir(fsblobstore, &mut dir2, "dir7").await;

    // Let's create a directory with lots of entries (so it'll use multiple nodes)
    for i in 0..100 {
        create_dir(fsblobstore, &mut dir2_dir6, &format!("dir{i}"))
            .await
            .async_drop()
            .await
            .unwrap();
        create_file(fsblobstore, &mut dir2_dir6, &format!("file{i}")).await;
        let target = (0..i)
            .map(|i| format!("pathcomponentforsymlink_{i}"))
            .join("/");
        create_symlink(
            fsblobstore,
            &mut dir2_dir6,
            &format!("symlink{i}"),
            &format!("symlink_target_{i}"),
        )
        .await;
    }
    assert!(
        dir2_dir6.num_nodes().await.unwrap() > 1_000,
        "If this fails, we need to create even more entries to make the directory large enough."
    );

    // Let's create a symlink with a very long path (so it'll use multiple nodes)
    let target = (0..1_000)
        .map(|i| format!("pathcomponentforsymlink_{i}"))
        .join("/");
    let mut symlink =
        create_symlink(fsblobstore, &mut dir2_dir7, &format!("symlink"), &target).await;
    assert!(
        symlink.num_nodes().await.unwrap() > 1_000,
        "If this fails, we need to make the target longer so it uses enough nodes."
    );

    // Let's create a file with lots of data (so it'll use multiple nodes)
    let mut file = create_file(fsblobstore, &mut dir2_dir7, "file").await;
    file.write(&data(16 * 1024, 0), 0).await.unwrap();
    assert!(
        file.num_nodes().await.unwrap() > 1_000,
        "If this fails, we need to make the data larger so it uses enough nodes."
    );

    dir2_dir7.async_drop().await.unwrap();
    dir2_dir6.async_drop().await.unwrap();
    dir1_dir3_dir5.async_drop().await.unwrap();
    dir1_dir4.async_drop().await.unwrap();
    dir1_dir3.async_drop().await.unwrap();
    dir2.async_drop().await.unwrap();
    dir1.async_drop().await.unwrap();
}

fn data(size: usize, seed: u64) -> Data {
    DataFixture::new(seed).get(size).into()
}
