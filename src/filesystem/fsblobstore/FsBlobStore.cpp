#include "FsBlobStore.h"
#include "FileBlob.h"
#include "DirBlob.h"
#include "SymlinkBlob.h"
#include "MagicNumbers.h"

namespace bf = boost::filesystem;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using blobstore::BlobStore;
using boost::none;

namespace cryfs {
namespace fsblobstore {

FsBlobStore::FsBlobStore(unique_ref<BlobStore> baseBlobStore): _baseBlobStore(std::move(baseBlobStore)) {
}

unique_ref<FileBlob> FsBlobStore::createFileBlob() {
    return FileBlob::InitializeEmptyFile(_baseBlobStore->create());
}

unique_ref<DirBlob> FsBlobStore::createDirBlob() {
    //TODO Passed in fsBlobStore should be ParallelAccessFsBlobStore later
    return DirBlob::InitializeEmptyDir(_baseBlobStore->create(), this);
}

unique_ref<SymlinkBlob> FsBlobStore::createSymlinkBlob(const bf::path &target) {
    return SymlinkBlob::InitializeSymlink(_baseBlobStore->create(), target);
}

boost::optional<unique_ref<FsBlob>> FsBlobStore::load(const blockstore::Key &key) {
    auto blob = _baseBlobStore->load(key);
    if (blob == none) {
        return none;
    }
    unsigned char magicNumber = FsBlob::magicNumber(**blob);
    if (magicNumber == MagicNumbers::FILE) {
        return unique_ref<FsBlob>(make_unique_ref<FileBlob>(std::move(*blob)));
    } else if (magicNumber == MagicNumbers::DIR) {
        //TODO Passed in fsBlobStore should be ParallelAccessFsBlobStore later
        return unique_ref<FsBlob>(make_unique_ref<DirBlob>(std::move(*blob), this));
    } else if (magicNumber == MagicNumbers::SYMLINK) {
        return unique_ref<FsBlob>(make_unique_ref<SymlinkBlob>(std::move(*blob)));
    } else {
        ASSERT(false, "Unknown magic number");
    }
}

void FsBlobStore::remove(cpputils::unique_ref<FsBlob> blob) {
    _baseBlobStore->remove(blob->releaseBaseBlob());
}

}
}