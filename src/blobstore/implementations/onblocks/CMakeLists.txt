add_library(blobstore_onblocks BlobOnBlocks.cpp BlobStoreOnBlocks.cpp impl/DataNode.cpp impl/DataLeafNode.cpp impl/DataInnerNode.cpp impl/DataNodeStore.cpp)

target_link_libraries(blobstore_onblocks blockstore_interface)
