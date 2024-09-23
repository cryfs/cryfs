#include "ParallelAccessFsBlobStore.h"
#include "blockstore/utils/BlockId.h"
#include "cpp-utils/assert/assert.h"
#include "cryfs/impl/filesystem/cachingfsblobstore/DirBlobRef.h"
#include "cryfs/impl/filesystem/cachingfsblobstore/FileBlobRef.h"
#include "cryfs/impl/filesystem/cachingfsblobstore/FsBlobRef.h"
#include "cryfs/impl/filesystem/cachingfsblobstore/SymlinkBlobRef.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/DirBlobRef.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/FileBlobRef.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/FsBlobRef.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/SymlinkBlobRef.h"
#include <boost/filesystem/path.hpp>
#include <utility>

namespace bf = boost::filesystem;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::optional;
using blockstore::BlockId;

namespace cryfs {
namespace parallelaccessfsblobstore {

optional<unique_ref<FsBlobRef>> ParallelAccessFsBlobStore::load(const BlockId &blockId) {
    return _parallelAccessStore.load(blockId, [] (cachingfsblobstore::FsBlobRef *blob) { // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )
        cachingfsblobstore::FileBlobRef *fileBlob = dynamic_cast<cachingfsblobstore::FileBlobRef*>(blob);
        if (fileBlob != nullptr) {
            return unique_ref<FsBlobRef>(make_unique_ref<FileBlobRef>(fileBlob));
        }
        cachingfsblobstore::DirBlobRef *dirBlob = dynamic_cast<cachingfsblobstore::DirBlobRef*>(blob);
        if (dirBlob != nullptr) {
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
    const BlockId blockId = blob->blockId();
    return _parallelAccessStore.add<DirBlobRef>(blockId, std::move(blob), [] (cachingfsblobstore::FsBlobRef *resource) {
        auto dirBlob = dynamic_cast<cachingfsblobstore::DirBlobRef*>(resource);
        ASSERT(dirBlob != nullptr, "Wrong resource given");
        return make_unique_ref<DirBlobRef>(dirBlob);
    });
}

unique_ref<FileBlobRef> ParallelAccessFsBlobStore::createFileBlob(const blockstore::BlockId &parent) {
    auto blob = _baseBlobStore->createFileBlob(parent);
    const BlockId blockId = blob->blockId();
    return _parallelAccessStore.add<FileBlobRef>(blockId, std::move(blob), [] (cachingfsblobstore::FsBlobRef *resource) {
        auto fileBlob = dynamic_cast<cachingfsblobstore::FileBlobRef*>(resource);
        ASSERT(fileBlob != nullptr, "Wrong resource given");
        return make_unique_ref<FileBlobRef>(fileBlob);
    });
}

unique_ref<SymlinkBlobRef> ParallelAccessFsBlobStore::createSymlinkBlob(const bf::path &target, const blockstore::BlockId &parent) {
    auto blob = _baseBlobStore->createSymlinkBlob(target, parent);
    const BlockId blockId = blob->blockId();
    return _parallelAccessStore.add<SymlinkBlobRef>(blockId, std::move(blob), [] (cachingfsblobstore::FsBlobRef *resource) {
        auto symlinkBlob = dynamic_cast<cachingfsblobstore::SymlinkBlobRef*>(resource);
        ASSERT(symlinkBlob != nullptr, "Wrong resource given");
        return make_unique_ref<SymlinkBlobRef>(symlinkBlob);
    });
}

}
}
