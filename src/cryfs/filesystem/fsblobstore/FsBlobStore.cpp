#include "FsBlobStore.h"
#include "FileBlob.h"
#include "DirBlob.h"
#include "SymlinkBlob.h"
#include "MagicNumbers.h"

namespace bf = boost::filesystem;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using blobstore::BlobStore;
using blockstore::Key;
using boost::none;
using std::function;

namespace cryfs {
namespace fsblobstore {

FsBlobStore::FsBlobStore(unique_ref<BlobStore> baseBlobStore): _baseBlobStore(std::move(baseBlobStore)) {
}

unique_ref<FileBlob> FsBlobStore::createFileBlob() {
    auto blob = _baseBlobStore->create();
    return FileBlob::InitializeEmptyFile(std::move(blob));
}

unique_ref<DirBlob> FsBlobStore::createDirBlob() {
    auto blob = _baseBlobStore->create();
    return DirBlob::InitializeEmptyDir(std::move(blob), _getLstatSize());
}

unique_ref<SymlinkBlob> FsBlobStore::createSymlinkBlob(const bf::path &target) {
    auto blob = _baseBlobStore->create();
    return SymlinkBlob::InitializeSymlink(std::move(blob), target);
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
        return unique_ref<FsBlob>(make_unique_ref<DirBlob>(std::move(*blob), _getLstatSize()));
    } else if (magicNumber == MagicNumbers::SYMLINK) {
        return unique_ref<FsBlob>(make_unique_ref<SymlinkBlob>(std::move(*blob)));
    } else {
        ASSERT(false, "Unknown magic number");
    }
}

void FsBlobStore::remove(cpputils::unique_ref<FsBlob> blob) {
    _baseBlobStore->remove(blob->releaseBaseBlob());
}

function<off_t (const Key &)> FsBlobStore::_getLstatSize() {
    return [this] (const Key &key) {
        auto blob = load(key);
        ASSERT(blob != none, "Blob not found");
        return (*blob)->lstat_size();
    };
}

}
}