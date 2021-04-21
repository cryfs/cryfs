#include "traversal.h"

#include <blobstore/implementations/onblocks/datanodestore/DataInnerNode.h>

using blockstore::BlockId;
using blockstore::BlockStore;
using cryfs::fsblobstore::FsBlobStore;
using cryfs::fsblobstore::DirBlob;
using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataInnerNode;
using cpputils::dynamic_pointer_move;

using std::vector;
using std::function;
using boost::none;

namespace cryfs_stats {

void forEachBlock(BlockStore* blockStore, const vector<function<void (const BlockId& blobId)>>& callbacks) {
    blockStore->forEachBlock([&callbacks] (const BlockId& blockId) {
        for(const auto& callback : callbacks) {
            callback(blockId);
        }
    });
}

// NOLINTNEXTLINE(misc-no-recursion)
void forEachReachableBlob(FsBlobStore* blobStore, const BlockId& rootId, const vector<function<void (const BlockId& blobId)>>& callbacks) {
    for (const auto& callback : callbacks) {
        callback(rootId);
    }

    auto rootBlob = blobStore->load(rootId);
    ASSERT(rootBlob != none, "Blob not found but referenced from directory entry");

    auto rootDir = dynamic_pointer_move<DirBlob>(*rootBlob);
    if (rootDir != none) {
        vector<fspp::Dir::Entry> children;
        children.reserve((*rootDir)->NumChildren());
        (*rootDir)->AppendChildrenTo(&children);

        for (const auto& child : children) {
            auto childEntry = (*rootDir)->GetChild(child.name);
            ASSERT(childEntry != none, "We just got this from the entry list, it must exist.");
            auto childId = childEntry->blockId();
            forEachReachableBlob(blobStore, childId, callbacks);
        }
    }
}

// NOLINTNEXTLINE(misc-no-recursion)
void forEachReachableBlockInBlob(DataNodeStore* nodeStore, const BlockId& rootId, const vector<function<void (const BlockId& blockId)>>& callbacks) {
    for (const auto& callback : callbacks) {
        callback(rootId);
    }

    auto node = nodeStore->load(rootId);
    auto innerNode = dynamic_pointer_move<DataInnerNode>(*node);
    if (innerNode != none) {
        for (uint32_t childIndex = 0; childIndex < (*innerNode)->numChildren(); ++childIndex) {
            auto childId = (*innerNode)->readChild(childIndex).blockId();
            forEachReachableBlockInBlob(nodeStore, childId, callbacks);
        }
    }
}

}
