#include "FsBlobStore.h"
#include "FileBlob.h"
#include "DirBlob.h"
#include "SymlinkBlob.h"
#include <cryfs/impl/config/CryConfigFile.h>
#include <cpp-utils/io/ProgressBar.h>
#include <cpp-utils/process/SignalCatcher.h>

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::SignalCatcher;
using blobstore::BlobStore;
using blockstore::BlockId;
using boost::none;
using std::vector;

namespace cryfs {
namespace fsblobstore {

boost::optional<unique_ref<FsBlob>> FsBlobStore::load(const blockstore::BlockId &blockId) {
    auto blob = _baseBlobStore->load(blockId);
    if (blob == none) {
        return none;
    }
    FsBlobView::BlobType blobType = FsBlobView::blobType(**blob);
    if (blobType == FsBlobView::BlobType::FILE) {
        return unique_ref<FsBlob>(make_unique_ref<FileBlob>(std::move(*blob)));
    } else if (blobType == FsBlobView::BlobType::DIR) {
        return unique_ref<FsBlob>(make_unique_ref<DirBlob>(std::move(*blob), _getLstatSize()));
    } else if (blobType == FsBlobView::BlobType::SYMLINK) {
        return unique_ref<FsBlob>(make_unique_ref<SymlinkBlob>(std::move(*blob)));
    } else {
        ASSERT(false, "Unknown magic number");
    }
}

#ifndef CRYFS_NO_COMPATIBILITY
    unique_ref<FsBlobStore> FsBlobStore::migrate(unique_ref<BlobStore> blobStore, const blockstore::BlockId &rootBlobId) {
        SignalCatcher signalCatcher;

        auto rootBlob = blobStore->load(rootBlobId);
        if (rootBlob == none) {
            throw std::runtime_error("Could not load root blob");
        }

        auto fsBlobStore = make_unique_ref<FsBlobStore>(std::move(blobStore));

        uint64_t migratedBlocks = 0;
        cpputils::ProgressBar progressbar("Migrating file system for conflict resolution features. This can take a while...", fsBlobStore->numBlocks());
        fsBlobStore->_migrate(std::move(*rootBlob), blockstore::BlockId::Null(), &signalCatcher, [&] (uint32_t numNodes) {
            migratedBlocks += numNodes;
            progressbar.update(migratedBlocks);
        });

        return fsBlobStore;
    }
    
    // NOLINTNEXTLINE(misc-no-recursion)
    void FsBlobStore::_migrate(unique_ref<blobstore::Blob> node, const blockstore::BlockId &parentId, SignalCatcher* signalCatcher, std::function<void(uint32_t numNodes)> perBlobCallback) {
        FsBlobView::migrate(node.get(), parentId);
        perBlobCallback(node->numNodes());
        if (FsBlobView::blobType(*node) == FsBlobView::BlobType::DIR) {
            DirBlob dir(std::move(node), _getLstatSize());
            vector<fspp::Dir::Entry> children;
            dir.AppendChildrenTo(&children);
            for (const auto &child : children) {
                if (signalCatcher->signal_occurred()) {
                    // on a SIGINT or SIGTERM, cancel migration but gracefully shutdown, i.e. call destructors.
                    throw std::runtime_error("Caught signal");
                }
                auto childEntry = dir.GetChild(child.name);
                ASSERT(childEntry != none, "Couldn't load child, although it was returned as a child in the list.");
                auto childBlob = _baseBlobStore->load(childEntry->blockId());
                ASSERT(childBlob != none, "Couldn't load child blob");
                _migrate(std::move(*childBlob), dir.blockId(), signalCatcher, perBlobCallback);
            }
        }
    }
#endif

}
}
