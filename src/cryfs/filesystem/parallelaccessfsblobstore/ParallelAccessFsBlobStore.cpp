#include "ParallelAccessFsBlobStore.h"
#include "ParallelAccessFsBlobStoreAdapter.h"
#include "../fsblobstore/FsBlobStore.h"

namespace bf = boost::filesystem;
using cryfs::cachingfsblobstore::CachingFsBlobStore;
using cryfs::fsblobstore::FsBlob;
using cryfs::fsblobstore::FileBlob;
using cryfs::fsblobstore::DirBlob;
using cryfs::fsblobstore::SymlinkBlob;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::optional;
using boost::none;
using blockstore::Key;

namespace cryfs {
namespace parallelaccessfsblobstore {

optional<unique_ref<FsBlobRef>> ParallelAccessFsBlobStore::load(const Key &key) {
    return _parallelAccessStore.load(key, [this] (cachingfsblobstore::FsBlobRef *blob) {
        cachingfsblobstore::FileBlobRef *fileBlob = dynamic_cast<cachingfsblobstore::FileBlobRef*>(blob);
        if (fileBlob != nullptr) {
            return unique_ref<FsBlobRef>(make_unique_ref<FileBlobRef>(fileBlob));
        }
        cachingfsblobstore::DirBlobRef *dirBlob = dynamic_cast<cachingfsblobstore::DirBlobRef*>(blob);
        if (dirBlob != nullptr) {
            dirBlob->setLstatSizeGetter(_getLstatSize());
            return unique_ref<FsBlobRef>(make_unique_ref<DirBlobRef>(dirBlob));
        }
        cachingfsblobstore::SymlinkBlobRef *symlinkBlob = dynamic_cast<cachingfsblobstore::SymlinkBlobRef*>(blob);
        if (symlinkBlob != nullptr) {
            return unique_ref<FsBlobRef>(make_unique_ref<SymlinkBlobRef>(symlinkBlob));
        }
        ASSERT(false, "Unknown blob type loaded");
    });
}

unique_ref<DirBlobRef> ParallelAccessFsBlobStore::createDirBlob() {
    auto blob = _baseBlobStore->createDirBlob();
    blob->setLstatSizeGetter(_getLstatSize());
    Key key = blob->key();
    return _parallelAccessStore.add<DirBlobRef>(key, std::move(blob), [] (cachingfsblobstore::FsBlobRef *resource) {
        auto dirBlob = dynamic_cast<cachingfsblobstore::DirBlobRef*>(resource);
        ASSERT(dirBlob != nullptr, "Wrong resource given");
        return make_unique_ref<DirBlobRef>(dirBlob);
    });
}

unique_ref<FileBlobRef> ParallelAccessFsBlobStore::createFileBlob() {
    auto blob = _baseBlobStore->createFileBlob();
    Key key = blob->key();
    return _parallelAccessStore.add<FileBlobRef>(key, std::move(blob), [] (cachingfsblobstore::FsBlobRef *resource) {
        auto fileBlob = dynamic_cast<cachingfsblobstore::FileBlobRef*>(resource);
        ASSERT(fileBlob != nullptr, "Wrong resource given");
        return make_unique_ref<FileBlobRef>(fileBlob);
    });
}

unique_ref<SymlinkBlobRef> ParallelAccessFsBlobStore::createSymlinkBlob(const bf::path &target) {
    auto blob = _baseBlobStore->createSymlinkBlob(target);
    Key key = blob->key();
    return _parallelAccessStore.add<SymlinkBlobRef>(key, std::move(blob), [] (cachingfsblobstore::FsBlobRef *resource) {
        auto symlinkBlob = dynamic_cast<cachingfsblobstore::SymlinkBlobRef*>(resource);
        ASSERT(symlinkBlob != nullptr, "Wrong resource given");
        return make_unique_ref<SymlinkBlobRef>(symlinkBlob);
    });
}

}
}
