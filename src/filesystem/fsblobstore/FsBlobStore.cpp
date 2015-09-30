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
using std::function;

namespace cryfs {
namespace fsblobstore {

FsBlobStore::FsBlobStore(unique_ref<BlobStore> baseBlobStore): _baseBlobStore(std::move(baseBlobStore)) {
}

unique_ref<FileBlob> FsBlobStore::createFileBlob() {
    auto blob = _baseBlobStore->create();
    auto key = blob->key();
    _openBlobs.lock(key);
    return FileBlob::InitializeEmptyFile(std::move(blob), freeLockFunction(key));
}

unique_ref<DirBlob> FsBlobStore::createDirBlob() {
    auto blob = _baseBlobStore->create();
    auto key = blob->key();
    _openBlobs.lock(key);
    //TODO Passed in fsBlobStore should be ParallelAccessFsBlobStore later
    return DirBlob::InitializeEmptyDir(std::move(blob), this, freeLockFunction(key));
}

unique_ref<SymlinkBlob> FsBlobStore::createSymlinkBlob(const bf::path &target) {
    auto blob = _baseBlobStore->create();
    auto key = blob->key();
    _openBlobs.lock(key);
    return SymlinkBlob::InitializeSymlink(std::move(blob), target, freeLockFunction(key));
}

boost::optional<unique_ref<FsBlob>> FsBlobStore::load(const blockstore::Key &key) {
    _openBlobs.lock(key);

    auto blob = _baseBlobStore->load(key);
    if (blob == none) {
        return none;
    }
    unsigned char magicNumber = FsBlob::magicNumber(**blob);
    if (magicNumber == MagicNumbers::FILE) {
        return unique_ref<FsBlob>(make_unique_ref<FileBlob>(std::move(*blob), freeLockFunction(key)));
    } else if (magicNumber == MagicNumbers::DIR) {
        //TODO Passed in fsBlobStore should be ParallelAccessFsBlobStore later
        return unique_ref<FsBlob>(make_unique_ref<DirBlob>(std::move(*blob), this, freeLockFunction(key)));
    } else if (magicNumber == MagicNumbers::SYMLINK) {
        return unique_ref<FsBlob>(make_unique_ref<SymlinkBlob>(std::move(*blob), freeLockFunction(key)));
    } else {
        ASSERT(false, "Unknown magic number");
    }
}

void FsBlobStore::remove(cpputils::unique_ref<FsBlob> blob) {
    _baseBlobStore->remove(blob->releaseBaseBlob());
}

function<void()> FsBlobStore::freeLockFunction(const blockstore::Key &key) {
    return [this, key] {
        _openBlobs.release(key);
    };
}

}
}