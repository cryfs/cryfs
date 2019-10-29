#include "FsBlobStore.h"
#include "FileBlob.h"
#include "DirBlob.h"
#include "SymlinkBlob.h"
#include <cpp-utils/io/ProgressBar.h>
#include <cpp-utils/process/SignalCatcher.h>

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::SignalCatcher;
using blobstore::BlobStore;
using blockstore::BlockId;
using boost::none;

namespace cryfs {
namespace fsblobstore {

boost::optional<unique_ref<FsBlob>> FsBlobStore::load(const blockstore::BlockId &blockId) {
    auto blob = _baseBlobStore->load(blockId);
    if (blob == none) {
        return none;
    }
    FsBlobView::BlobType blobType = FsBlobView::blobType(**blob);
    if (blobType == FsBlobView::BlobType::FILE) {
        return unique_ref<FsBlob>(make_unique_ref<FileBlob>(std::move(*blob), _timestampUpdateBehavior));
    } else if (blobType == FsBlobView::BlobType::DIR) {
        return unique_ref<FsBlob>(make_unique_ref<DirBlob>(std::move(*blob), _timestampUpdateBehavior));
    } else if (blobType == FsBlobView::BlobType::SYMLINK) {
        return unique_ref<FsBlob>(make_unique_ref<SymlinkBlob>(std::move(*blob), _timestampUpdateBehavior));
    } else {
        ASSERT(false, "Unknown magic number");
    }
}

#ifndef CRYFS_NO_COMPATIBILITY
    unique_ref<FsBlobStore> FsBlobStore::migrate(unique_ref<BlobStore> blobStore, const blockstore::BlockId &rootBlobId,
            const TimestampUpdateBehavior& behavior) {
        SignalCatcher signalCatcher;

        auto rootBlob = blobStore->load(rootBlobId);
        if (rootBlob == none) {
            throw std::runtime_error("Could not load root blob");
        }

        auto fsBlobStore = make_unique_ref<FsBlobStore>(std::move(blobStore), behavior);

        uint64_t migratedBlocks = 0;
        cpputils::ProgressBar progressbar("Migrating file system for conflict resolution features. This can take a while...", fsBlobStore->numBlocks());
        fsBlobStore->_migrate(std::move(*rootBlob), FsBlobView::Metadata::rootMetaData(), FsBlobView::BlobType::DIR, &signalCatcher, [&] (uint32_t numNodes) {
            migratedBlocks += numNodes;
            progressbar.update(migratedBlocks);
        });

        return fsBlobStore;
    }

    void FsBlobStore::_migrate(unique_ref<blobstore::Blob> node, const FsBlobView::Metadata& metadata, FsBlobView::BlobType type, SignalCatcher* signalCatcher, const std::function<void(uint32_t numNodes)>& perBlobCallback) {
        auto childEntries = FsBlobView::migrate(node.get(), metadata, type);
        perBlobCallback(node->numNodes());
        for (const auto& e : childEntries) {
          auto childBlob = _baseBlobStore->load(e._blockId);
          ASSERT(childBlob != none, "Couldn't load child blob");
          // we start with 1 link, directories will be handled inside the _migrate function and
          // the size of 0 bytes will always be set dynamically.
          FsBlobView::Metadata m(1u, e._mode, e._uid, e._gid, fspp::num_bytes_t(0), e._lastAccessTime, e._lastModificationTime, e._lastMetadataChangeTime);
          _migrate(std::move(*childBlob), m, e._type, signalCatcher, perBlobCallback);
        }
    }
#endif

}
}
