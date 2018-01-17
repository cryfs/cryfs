#include "parallelaccessdatatreestore/DataTreeRef.h"
#include "parallelaccessdatatreestore/ParallelAccessDataTreeStore.h"
#include <blockstore/implementations/threadsafe/ThreadsafeBlockStore.h>
#include "datanodestore/DataLeafNode.h"
#include "datanodestore/DataNodeStore.h"
#include "datatreestore/DataTreeStore.h"
#include "datatreestore/DataTree.h"
#include "BlobStoreOnBlocks.h"
#include "BlobOnBlocks.h"
#include <cpp-utils/pointer/cast.h>
#include <cpp-utils/assert/assert.h>

using cpputils::unique_ref;
using cpputils::make_unique_ref;

using blockstore::BlockStore;
using blockstore::threadsafe::ThreadsafeBlockStore;
using blockstore::BlockId;
using cpputils::dynamic_pointer_move;
using boost::optional;
using boost::none;

namespace blobstore {
namespace onblocks {

using datanodestore::DataNodeStore;
using datatreestore::DataTreeStore;
using parallelaccessdatatreestore::ParallelAccessDataTreeStore;

BlobStoreOnBlocks::BlobStoreOnBlocks(unique_ref<BlockStore> blockStore, uint64_t physicalBlocksizeBytes)
        : _dataTreeStore(make_unique_ref<ParallelAccessDataTreeStore>(make_unique_ref<DataTreeStore>(make_unique_ref<DataNodeStore>(make_unique_ref<ThreadsafeBlockStore>(std::move(blockStore)), physicalBlocksizeBytes)))) {
}

BlobStoreOnBlocks::~BlobStoreOnBlocks() {
}

unique_ref<Blob> BlobStoreOnBlocks::create() {
    return make_unique_ref<BlobOnBlocks>(_dataTreeStore->createNewTree());
}

optional<unique_ref<Blob>> BlobStoreOnBlocks::load(const BlockId &blockId) {
    auto tree = _dataTreeStore->load(blockId);
    if (tree == none) {
        return none;
    }
    return optional<unique_ref<Blob>>(make_unique_ref<BlobOnBlocks>(std::move(*tree)));
}

void BlobStoreOnBlocks::remove(unique_ref<Blob> blob) {
    auto _blob = dynamic_pointer_move<BlobOnBlocks>(blob);
    ASSERT(_blob != none, "Passed Blob in BlobStoreOnBlocks::remove() is not a BlobOnBlocks.");
    _dataTreeStore->remove((*_blob)->releaseTree());
}

void BlobStoreOnBlocks::remove(const BlockId &blockId) {
    _dataTreeStore->remove(blockId);
}

uint64_t BlobStoreOnBlocks::virtualBlocksizeBytes() const {
    return _dataTreeStore->virtualBlocksizeBytes();
}

uint64_t BlobStoreOnBlocks::numBlocks() const {
    return _dataTreeStore->numNodes();
}

uint64_t BlobStoreOnBlocks::estimateSpaceForNumBlocksLeft() const {
    return _dataTreeStore->estimateSpaceForNumNodesLeft();
}


}
}
