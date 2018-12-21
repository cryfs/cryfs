#include "ParallelAccessFsBlobStore.h"
#include "ParallelAccessFsBlobStoreAdapter.h"
#include "cryfs/impl/filesystem/fsblobstore/FsBlobStore.h"

namespace bf = boost::filesystem;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::optional;
using blockstore::BlockId;

namespace cryfs {
namespace parallelaccessfsblobstore {

optional<unique_ref<FsBlobRef>> ParallelAccessFsBlobStore::load(const BlockId &blockId) {
    return _parallelAccessStore.load(blockId, [this] (cachingfsblobstore::FsBlobRef *blob) { // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )
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

unique_ref<DirBlobRef> ParallelAccessFsBlobStore::createDirBlob(const blockstore::BlockId &parent) {
    auto blob = _baseBlobStore->createDirBlob(parent);
    blob->setLstatSizeGetter(_getLstatSize());
    BlockId blockId = blob->blockId();
    return _parallelAccessStore.add<DirBlobRef>(blockId, std::move(blob), [] (cachingfsblobstore::FsBlobRef *resource) {
        auto dirBlob = dynamic_cast<cachingfsblobstore::DirBlobRef*>(resource);
        ASSERT(dirBlob != nullptr, "Wrong resource given");
        return make_unique_ref<DirBlobRef>(dirBlob);
    });
}

unique_ref<FileBlobRef> ParallelAccessFsBlobStore::createFileBlob(const blockstore::BlockId &parent) {
    auto blob = _baseBlobStore->createFileBlob(parent);
    BlockId blockId = blob->blockId();
    return _parallelAccessStore.add<FileBlobRef>(blockId, std::move(blob), [] (cachingfsblobstore::FsBlobRef *resource) {
        auto fileBlob = dynamic_cast<cachingfsblobstore::FileBlobRef*>(resource);
        ASSERT(fileBlob != nullptr, "Wrong resource given");
        return make_unique_ref<FileBlobRef>(fileBlob);
    });
}

unique_ref<SymlinkBlobRef> ParallelAccessFsBlobStore::createSymlinkBlob(const bf::path &target, const blockstore::BlockId &parent) {
    auto blob = _baseBlobStore->createSymlinkBlob(target, parent);
    BlockId blockId = blob->blockId();
    return _parallelAccessStore.add<SymlinkBlobRef>(blockId, std::move(blob), [] (cachingfsblobstore::FsBlobRef *resource) {
        auto symlinkBlob = dynamic_cast<cachingfsblobstore::SymlinkBlobRef*>(resource);
        ASSERT(symlinkBlob != nullptr, "Wrong resource given");
        return make_unique_ref<SymlinkBlobRef>(symlinkBlob);
    });
}

}
}
