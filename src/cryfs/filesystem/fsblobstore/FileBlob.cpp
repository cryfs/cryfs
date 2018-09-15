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

fspp::num_bytes_t FileBlob::read(void *target, fspp::num_bytes_t offset, fspp::num_bytes_t count) const {
  return fspp::num_bytes_t(baseBlob().tryRead(target, offset.value(), count.value()));
}

void FileBlob::write(const void *source, fspp::num_bytes_t offset, fspp::num_bytes_t count) {
  baseBlob().write(source, offset.value(), count.value());
}

void FileBlob::flush() {
  baseBlob().flush();
}

void FileBlob::resize(fspp::num_bytes_t size) {
  baseBlob().resize(size.value());
}

fspp::num_bytes_t FileBlob::lstat_size() const {
  return size();
}

fspp::num_bytes_t FileBlob::size() const {
  return fspp::num_bytes_t(baseBlob().size());
}

}
}

