//! Tests where there are nodes that aren't referenced from anywhere

use cryfs_blockstore::BlockId;
use cryfs_check::{
    CorruptedError, MaybeBlobReferenceWithId, NodeAndBlobReference, NodeInfoAsSeenByLookingAtNode,
    NodeMissingError, NodeUnreferencedError,
};
use cryfs_utils::{
    data::Data, testutils::asserts::assert_unordered_vec_eq, testutils::data_fixture::DataFixture,
};
use std::num::NonZeroU8;

mod common;
use common::fixture::FilesystemFixture;

fn block_id(seed: u64) -> BlockId {
    BlockId::from_slice(&DataFixture::new(seed).get(16)).unwrap()
}
#[tokio::test(flavor = "multi_thread")]
async fn leaf_node_unreferenced() {
    let (fs_fixture, _some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let node_id = fs_fixture
        .update_nodestore(|nodestore| {
            Box::pin(async move {
                *nodestore
                    .create_new_leaf_node(&Data::empty())
                    .await
                    .unwrap()
                    .block_id()
            })
        })
        .await;

    let expected_errors: Vec<_> = vec![CorruptedError::NodeUnreferenced(NodeUnreferencedError {
        node_id,
        node_info: NodeInfoAsSeenByLookingAtNode::LeafNode,
    })];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_eq!(expected_errors, errors,);
}

#[tokio::test(flavor = "multi_thread")]
async fn single_inner_node_unreferenced() {
    const DEPTH: u8 = 3;
    let (fs_fixture, _some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let node_id = fs_fixture
        .update_nodestore(|nodestore| {
            Box::pin(async move {
                *nodestore
                    .create_new_inner_node(DEPTH, &[block_id(0), block_id(1)])
                    .await
                    .unwrap()
                    .block_id()
            })
        })
        .await;

    let referenced_as = NodeAndBlobReference::NonRootInnerNode {
        belongs_to_blob: MaybeBlobReferenceWithId::UnreachableFromFilesystemRoot,
        depth: NonZeroU8::new(DEPTH - 1).unwrap(),
        parent_id: node_id,
    };

    let expected_errors: Vec<_> = vec![
        NodeUnreferencedError {
            node_id,
            node_info: NodeInfoAsSeenByLookingAtNode::InnerNode {
                depth: NonZeroU8::new(DEPTH).unwrap(),
            },
        }
        .into(),
        NodeMissingError {
            node_id: block_id(0),
            referenced_as: [referenced_as.clone()].into_iter().collect(),
        }
        .into(),
        NodeMissingError {
            node_id: block_id(1),
            referenced_as: [referenced_as].into_iter().collect(),
        }
        .into(),
    ];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[tokio::test(flavor = "multi_thread")]
async fn inner_node_with_subtree_unreferenced() {
    let (fs_fixture, _some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let node_id = fs_fixture
        .update_nodestore(|nodestore| {
            Box::pin(async move {
                let leaf1 = *nodestore
                    .create_new_leaf_node(&Data::empty())
                    .await
                    .unwrap()
                    .block_id();
                let leaf2 = *nodestore
                    .create_new_leaf_node(&Data::empty())
                    .await
                    .unwrap()
                    .block_id();
                let leaf3 = *nodestore
                    .create_new_leaf_node(&Data::empty())
                    .await
                    .unwrap()
                    .block_id();
                let leaf4 = *nodestore
                    .create_new_leaf_node(&Data::empty())
                    .await
                    .unwrap()
                    .block_id();
                let inner1 = *nodestore
                    .create_new_inner_node(1, &[leaf1, leaf2])
                    .await
                    .unwrap()
                    .block_id();
                let inner2 = *nodestore
                    .create_new_inner_node(1, &[leaf3, leaf4])
                    .await
                    .unwrap()
                    .block_id();

                *nodestore
                    .create_new_inner_node(2, &[inner1, inner2])
                    .await
                    .unwrap()
                    .block_id()
            })
        })
        .await;

    let expected_errors: Vec<_> = vec![NodeUnreferencedError {
        node_id,
        node_info: NodeInfoAsSeenByLookingAtNode::InnerNode {
            depth: NonZeroU8::new(2).unwrap(),
        },
    }
    .into()];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}