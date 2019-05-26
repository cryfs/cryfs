#pragma once
#ifndef CRYFS_STATS_TRAVERSAL_H
#define CRYFS_STATS_TRAVERSAL_H

#include <vector>
#include <functional>
#include <blockstore/interface/Block.h>
#include <blobstore/implementations/onblocks/datanodestore/DataNodeStore.h>
#include <cryfs/impl/filesystem/fsblobstore/FsBlobStore.h>

namespace cryfs_stats {

    // Call the callbacks on each existing block, whether it is connected or orphaned
    void forEachBlock(blockstore::BlockStore* blockStore, const std::vector<std::function<void (const blockstore::BlockId& blobId)>>& callbacks);

    // Call the callbacks on each existing blob that is reachable from the root blob, i.e. not orphaned
    void forEachReachableBlob(cryfs::fsblobstore::FsBlobStore* blobStore, const blockstore::BlockId& rootId, const std::vector<std::function<void (const blockstore::BlockId& blobId)>>& callbacks);

    // Call the callbacks on each block that is reachable from the given blob root, i.e. belongs to this blob.
    void forEachReachableBlockInBlob(blobstore::onblocks::datanodestore::DataNodeStore* nodeStore, const blockstore::BlockId& rootId, const std::vector<std::function<void (const blockstore::BlockId& blockId)>>& callbacks);

}

#endif
