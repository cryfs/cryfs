#include "FileBlob.h"

#include "MagicNumbers.h"
#include <messmer/blockstore/utils/Key.h>
#include <cassert>

using blobstore::Blob;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using blockstore::Key;

namespace cryfs {
namespace fsblobstore {

FileBlob::FileBlob(unique_ref<Blob> blob)
: FsBlob(std::move(blob)) {
}

unique_ref<FileBlob> FileBlob::InitializeEmptyFile(unique_ref<Blob> blob) {
  InitializeBlobWithMagicNumber(blob.get(), MagicNumbers::FILE);
  return make_unique_ref<FileBlob>(std::move(blob));
}

ssize_t FileBlob::read(void *target, uint64_t offset, uint64_t count) const {
  return baseBlob().tryRead(target, offset + 1, count);
}

void FileBlob::write(const void *source, uint64_t offset, uint64_t count) {
  baseBlob().write(source, offset + 1, count);
}

void FileBlob::flush() {
  baseBlob().flush();
}

void FileBlob::resize(off_t size) {
  baseBlob().resize(size+1);
}

off_t FileBlob::lstat_size() const {
  return size();
}

off_t FileBlob::size() const {
  return baseBlob().size()-1;
}

}
}

