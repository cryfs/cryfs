#include "FsBlobStore.h"
#include "FileBlob.h"
#include "DirBlob.h"
#include "SymlinkBlob.h"
#include <cryfs/config/CryConfigFile.h>
#include <cpp-utils/process/SignalCatcher.h>

using cpputils::unique_ref;
using cpputils::make_unique_ref;
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
    unique_ref<FsBlobStore> FsBlobStore::migrateIfNeeded(unique_ref<BlobStore> blobStore, const blockstore::BlockId &rootBlobId) {
        auto rootBlob = blobStore->load(rootBlobId);
        ASSERT(rootBlob != none, "Could not load root blob");
        uint16_t format = FsBlobView::getFormatVersionHeader(**rootBlob);

        auto fsBlobStore = make_unique_ref<FsBlobStore>(std::move(blobStore));
        if (format == 0) {
            // migration needed
            std::cout << "Migrating file system for conflict resolution features. Please don't interrupt this process. This can take a while..." << std::flush;
            fsBlobStore->_migrate(std::move(*rootBlob), blockstore::BlockId::Null());
            std::cout << "done" << std::endl;
        }
        return fsBlobStore;
    }

    void FsBlobStore::_migrate(unique_ref<blobstore::Blob> node, const blockstore::BlockId &parentId) {
        FsBlobView::migrate(node.get(), parentId);
        if (FsBlobView::blobType(*node) == FsBlobView::BlobType::DIR) {
            DirBlob dir(std::move(node), _getLstatSize());
            vector<fspp::Dir::Entry> children;
            dir.AppendChildrenTo(&children);
            for (const auto &child : children) {
                auto childEntry = dir.GetChild(child.name);
                ASSERT(childEntry != none, "Couldn't load child, although it was returned as a child in the lsit.");
                auto childBlob = _baseBlobStore->load(childEntry->blockId());
                ASSERT(childBlob != none, "Couldn't load child blob");
                _migrate(std::move(*childBlob), dir.blockId());
            }
        }
    }
#endif

}
}
