#include "CachingFsBlobStore.h"
#include "../fsblobstore/FsBlobStore.h"

namespace bf = boost::filesystem;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::dynamic_pointer_move;
using blobstore::BlobStore;
using blockstore::Key;
using boost::optional;
using boost::none;
using std::function;
using cryfs::fsblobstore::FsBlobStore;
using cryfs::fsblobstore::FsBlob;
using cryfs::fsblobstore::FileBlob;
using cryfs::fsblobstore::DirBlob;
using cryfs::fsblobstore::SymlinkBlob;

namespace cryfs {
namespace cachingfsblobstore {

    CachingFsBlobStore::CachingFsBlobStore(unique_ref<FsBlobStore> baseBlobStore)
        : _baseBlobStore(std::move(baseBlobStore)), _cache() {
    }

    CachingFsBlobStore::~CachingFsBlobStore() {
    }

    unique_ref<FileBlobRef> CachingFsBlobStore::createFileBlob() {
        // This already creates the file blob in the underlying blobstore.
        // We could also cache this operation, but that is more complicated (blockstore::CachingBlockStore does it)
        // and probably not worth it here.
        return make_unique_ref<FileBlobRef>(_baseBlobStore->createFileBlob(), this);
    }

    unique_ref<DirBlobRef> CachingFsBlobStore::createDirBlob() {
        // This already creates the file blob in the underlying blobstore.
        // We could also cache this operation, but that is more complicated (blockstore::CachingBlockStore does it)
        // and probably not worth it here.
        return make_unique_ref<DirBlobRef>(_baseBlobStore->createDirBlob(), this);
    }

    unique_ref<SymlinkBlobRef> CachingFsBlobStore::createSymlinkBlob(const bf::path &target) {
        // This already creates the file blob in the underlying blobstore.
        // We could also cache this operation, but that is more complicated (blockstore::CachingBlockStore does it)
        // and probably not worth it here.
        return make_unique_ref<SymlinkBlobRef>(_baseBlobStore->createSymlinkBlob(target), this);
    }

    optional<unique_ref<FsBlobRef>> CachingFsBlobStore::load(const Key &key) {
        auto fromCache = _cache.pop(key);
        if (fromCache != none) {
            return _makeRef(std::move(*fromCache));
        }
        auto fromBaseStore = _baseBlobStore->load(key);
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

    void CachingFsBlobStore::remove(unique_ref<FsBlobRef> blob) {
        auto baseBlob = blob->releaseBaseBlob();
        return _baseBlobStore->remove(std::move(baseBlob));
    }

    void CachingFsBlobStore::releaseForCache(cpputils::unique_ref<fsblobstore::FsBlob> baseBlob) {
        Key key = baseBlob->key();
        _cache.push(key, std::move(baseBlob));
    }

}
}
