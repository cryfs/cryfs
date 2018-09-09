#include "FileBlob.h"

#include <blockstore/utils/BlockId.h>
#include <cassert>

using blobstore::Blob;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using blockstore::BlockId;

namespace cryfs {
namespace fsblobstore {

FileBlob::FileBlob(unique_ref<Blob> blob)
: FsBlob(std::move(blob)) {
  ASSERT(baseBlob().blobType() == FsBlobView::BlobType::FILE, "Loaded blob is not a file");
}

unique_ref<FileBlob> FileBlob::InitializeEmptyFile(unique_ref<Blob> blob, const blockstore::BlockId &parent) {
  InitializeBlob(blob.get(), FsBlobView::BlobType::FILE, parent);
  return make_unique_ref<FileBlob>(std::move(blob));
}

size_t FileBlob::read(void *target, uint64_t offset, uint64_t count) const {
  return baseBlob().tryRead(target, offset, count);
}

void FileBlob::write(const void *source, uint64_t offset, uint64_t count) {
  baseBlob().write(source, offset, count);
}

void FileBlob::flush() {
  baseBlob().flush();
}

void FileBlob::resize(off_t size) {
  baseBlob().resize(size);
}

off_t FileBlob::lstat_size() const {
  return size();
}

off_t FileBlob::size() const {
  return baseBlob().size();
}

}
}

