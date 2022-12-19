#include "traversal.h"

using blockstore::BlockId;
using blockstore::BlockStore;
using cryfs::fsblobstore::rust::RustDirBlob;
using cryfs::fsblobstore::rust::RustFsBlobStore;
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
void forEachReachableBlob(RustFsBlobStore* blobStore, const BlockId& rootId, const vector<function<void (const BlockId& blobId)>>& callbacks) {
    for (const auto& callback : callbacks) {
        callback(rootId);
    }

    auto rootBlob = blobStore->load(rootId);
    ASSERT(rootBlob != none, "Blob not found but referenced from directory entry");

    if ((*rootBlob)->isDir()) {
        auto rootDir = std::move(**rootBlob).asDir();
        vector<fspp::Dir::Entry> children;
        children.reserve(rootDir->NumChildren());
        rootDir->AppendChildrenTo(&children);

        for (const auto& child : children) {
            auto childEntry = rootDir->GetChild(child.name);
            ASSERT(childEntry != none, "We just got this from the entry list, it must exist.");
            auto childId = (*childEntry)->blockId();
            forEachReachableBlob(blobStore, childId, callbacks);
        }
    }
}

// NOLINTNEXTLINE(misc-no-recursion)
void forEachReachableBlockInBlob(RustFsBlobStore* blobStore, const BlockId& rootId, const vector<function<void (const BlockId& blockId)>>& callbacks) {
    auto node = blobStore->load(rootId);
    ASSERT(node != none, "Blob not found");
    auto blocks = (*node)->allBlocks();
    for (const auto& block : blocks) {
        for (const auto& callback : callbacks) {
            callback(block);
        }
    }
}

}
