#include "CachingFsBlobStore.h"
#include "cryfs/impl/filesystem/fsblobstore/FsBlobStore.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::dynamic_pointer_move;
using blockstore::BlockId;
using boost::optional;
using boost::none;
using cryfs::fsblobstore::FsBlob;
using cryfs::fsblobstore::FileBlob;
using cryfs::fsblobstore::DirBlob;
using cryfs::fsblobstore::SymlinkBlob;

namespace cryfs {
namespace cachingfsblobstore {

    constexpr double CachingFsBlobStore::MAX_LIFETIME_SEC;

    optional<unique_ref<FsBlobRef>> CachingFsBlobStore::load(const BlockId &blockId) {
        auto fromCache = _cache.pop(blockId);
        if (fromCache != none) {
            return _makeRef(std::move(*fromCache));
        }
        auto fromBaseStore = _baseBlobStore->load(blockId);
        if (fromBaseStore != none) {
            return _makeRef(std::move(*fromBaseStore));
        }
        return none;
    }

    unique_ref<FsBlobRef> CachingFsBlobStore::_makeRef(unique_ref<FsBlob> baseBlob) {
        auto fileBlob = dynamic_pointer_move<FileBlob>(baseBlob);
        if (fileBlob != none) {
            return make_unique_ref<FileBlobRef>(std::move(*fileBlob), this);
        }
        auto dirBlob = dynamic_pointer_move<DirBlob>(baseBlob);
        if (dirBlob != none) {
            return make_unique_ref<DirBlobRef>(std::move(*dirBlob), this);
        }
        auto symlinkBlob = dynamic_pointer_move<SymlinkBlob>(baseBlob);
        if (symlinkBlob != none) {
            return make_unique_ref<SymlinkBlobRef>(std::move(*symlinkBlob), this);
        }
        ASSERT(false, "Unknown blob type");
    }
}
}
