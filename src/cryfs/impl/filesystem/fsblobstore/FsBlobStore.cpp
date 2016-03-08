#include "FsBlobStore.h"
#include "FileBlob.h"
#include "DirBlob.h"
#include "SymlinkBlob.h"

namespace bf = boost::filesystem;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using blobstore::BlobStore;
using blockstore::Key;
using boost::none;
using std::function;

namespace cryfs {
namespace fsblobstore {

boost::optional<unique_ref<FsBlob>> FsBlobStore::load(const blockstore::Key &key) {
    auto blob = _baseBlobStore->load(key);
    if (blob == none) {
        return none;
    }
    FsBlobView::BlobType blobType = FsBlobView::blobType(**blob);
    if (blobType == FsBlobView::BlobType::FILE) {
        return unique_ref<FsBlob>(make_unique_ref<FileBlob>(std::move(*blob)));
    } else if (blobType == FsBlobView::BlobType::DIR) {
        return unique_ref<FsBlob>(make_unique_ref<DirBlob>(this, std::move(*blob), _getLstatSize()));
    } else if (blobType == FsBlobView::BlobType::SYMLINK) {
        return unique_ref<FsBlob>(make_unique_ref<SymlinkBlob>(std::move(*blob)));
    } else {
        ASSERT(false, "Unknown magic number");
    }
}

}
}