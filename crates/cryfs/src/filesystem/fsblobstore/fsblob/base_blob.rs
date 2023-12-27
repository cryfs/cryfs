use anyhow::{ensure, Result};
use binary_layout::Field;
use futures::stream::BoxStream;
use std::fmt::Debug;

use super::layout::{self, FORMAT_VERSION_HEADER};
use cryfs_blobstore::{Blob, BlobId, BlobStore, BlobStoreOnBlocks, DataNode, LoadNodeError};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_utils::data::Data;

pub struct BaseBlob<'a, B>
where
    B: BlobStore + Debug + 'a,
{
    blob: B::ConcreteBlob<'a>,
    header_cache: layout::fsblob_header::View<Data>,
}

impl<'a, B> BaseBlob<'a, BlobStoreOnBlocks<B>>
where
    B: BlockStore + Send + Sync,
{
    pub fn load_all_nodes(self) -> BoxStream<'a, Result<DataNode<B>, LoadNodeError>> {
        self.blob.load_all_nodes()
    }
}

impl<'a, B> BaseBlob<'a, B>
where
    B: BlobStore + Debug + 'a,
{
    pub async fn parse(mut blob: B::ConcreteBlob<'a>) -> Result<BaseBlob<'a, B>> {
        // TODO No need to zero-initialize
        let mut header = vec![0; layout::fsblob_header::SIZE.unwrap()];
        blob.read(&mut header, 0).await?;
        let header_cache = layout::fsblob_header::View::new(header.into());
        ensure!(
            header_cache.format_version_header().read() == FORMAT_VERSION_HEADER,
            "Loaded FsBlob with format version {} but current version is {}",
            header_cache.format_version_header().read(),
            FORMAT_VERSION_HEADER
        );
        Ok(Self { blob, header_cache })
    }

    pub async fn try_create_with_id(
        blob_id: &BlobId,
        blobstore: &'a B,
        blob_type: layout::BlobType,
        parent: &BlobId,
        data: &[u8],
    ) -> Result<Option<<B as BlobStore>::ConcreteBlob<'a>>> {
        let blob_data = create_data_for_new_blob(blob_type, parent, data);

        // TODO Directly creating the blob with the data would probably be faster
        // than first creating it empty and then writing to it
        let Some(mut blob) = blobstore.try_create(blob_id).await? else {
            return Ok(None);
        };
        blob.write(&blob_data, 0).await?;
        Ok(Some(blob))
    }

    pub async fn create(
        blobstore: &'a B,
        blob_type: layout::BlobType,
        parent: &BlobId,
        data: &[u8],
    ) -> Result<BaseBlob<'a, B>> {
        let blob_data = create_data_for_new_blob(blob_type, parent, data);

        // TODO Directly creating the blob with the data would probably be faster
        // than first creating it empty and then writing to it
        let mut blob = blobstore.create().await?;
        blob.write(&blob_data, 0).await?;

        let mut header_cache = blob_data;
        header_cache.shrink_to_subregion(..layout::fsblob_header::SIZE.unwrap());

        Ok(Self {
            blob,
            header_cache: layout::fsblob_header::View::new(header_cache),
        })
    }

    pub fn blob_id(&self) -> BlobId {
        self.blob.id()
    }

    pub fn blob_type(&self) -> layout::BlobType {
        self.header_cache.blob_type().read()
    }

    pub fn parent(&self) -> BlobId {
        BlobId::from_array(self.header_cache.parent())
    }

    pub async fn set_parent(&mut self, new_parent: &BlobId) -> Result<()> {
        self.blob
            .write(
                new_parent.data(),
                layout::fsblob_header::parent::OFFSET as u64,
            )
            .await?;
        self.header_cache
            .parent_mut()
            .copy_from_slice(new_parent.data());
        Ok(())
    }

    pub async fn num_data_bytes(&mut self) -> Result<u64> {
        // TODO Make self parameter non-mut?
        Ok(self.blob.num_bytes().await? - layout::fsblob_header::SIZE.unwrap() as u64)
    }

    pub async fn resize_data(&mut self, new_num_bytes: u64) -> Result<()> {
        self.blob
            .resize(new_num_bytes + layout::fsblob_header::SIZE.unwrap() as u64)
            .await
    }

    pub async fn try_read_data(&mut self, target: &mut [u8], offset: u64) -> Result<usize> {
        // TODO Make self parameter non-mut?
        self.blob
            .try_read(target, offset + layout::fsblob_header::SIZE.unwrap() as u64)
            .await
    }

    pub async fn read_all_data(&mut self) -> Result<Data> {
        // TODO We should probably enforce a max size for the read so we don't block when a file system is bad
        //      This is only used for symlink blobs right now and those aren't supposed to be that large.
        let mut data = self.blob.read_all().await?;
        ensure!(
            data.len() >= layout::fsblob_header::SIZE.unwrap(),
            "Blob is too small to contain a header"
        );
        data.shrink_to_subregion(layout::fsblob_header::SIZE.unwrap()..);
        Ok(data)
    }

    pub async fn write_data(&mut self, source: &[u8], offset: u64) -> Result<()> {
        self.blob
            .write(source, offset + layout::fsblob_header::SIZE.unwrap() as u64)
            .await
    }

    pub async fn flush(&mut self) -> Result<()> {
        self.blob.flush().await
    }

    pub async fn remove(self) -> Result<()> {
        self.blob.remove().await
    }

    pub fn all_blocks(&self) -> Result<BoxStream<'_, Result<BlockId>>> {
        self.blob.all_blocks()
    }

    #[cfg(any(test, feature = "testutils"))]
    pub async fn num_nodes(&mut self) -> Result<u64> {
        self.blob.num_nodes().await
    }
}

fn create_data_for_new_blob(blob_type: layout::BlobType, parent: &BlobId, data: &[u8]) -> Data {
    // TODO No need to zero-fill header
    let blob_data: Data = vec![0; layout::fsblob_header::SIZE.unwrap() + data.len()].into();
    let mut view = layout::fsblob::View::new(blob_data);
    view.header_mut()
        .format_version_header_mut()
        .write(layout::FORMAT_VERSION_HEADER);
    view.header_mut().blob_type_mut().write(blob_type);
    view.header_mut()
        .parent_mut()
        .copy_from_slice(parent.data());
    view.data_mut().copy_from_slice(data);
    view.into_storage()
}
