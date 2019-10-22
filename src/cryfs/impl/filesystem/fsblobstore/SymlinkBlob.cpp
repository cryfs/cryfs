#include "SymlinkBlob.h"

#include <blockstore/utils/BlockId.h>

using std::string;
using blobstore::Blob;
using cpputils::unique_ref;
using cpputils::make_unique_ref;

namespace bf = boost::filesystem;

namespace cryfs {
namespace fsblobstore {

SymlinkBlob::SymlinkBlob(unique_ref<Blob> blob)
: FsBlob(std::move(blob)), _target(_readTargetFromBlob(baseBlob())) {
  ASSERT(baseBlob().blobType() == FsBlobView::BlobType::SYMLINK, "Loaded blob is not a symlink");
}

unique_ref<SymlinkBlob> SymlinkBlob::InitializeSymlink(unique_ref<Blob> blob, const bf::path &target, const FsBlobView::Metadata &meta) {
  InitializeBlob(blob.get(),  meta, FsBlobView::BlobType::SYMLINK);
  FsBlobView symlinkBlobView(std::move(blob));
  const string& targetStr = target.string();
  symlinkBlobView.resize(targetStr.size());
  symlinkBlobView.write(targetStr.c_str(), 0, targetStr.size());
  return make_unique_ref<SymlinkBlob>(symlinkBlobView.releaseBaseBlob());
}

bf::path SymlinkBlob::_readTargetFromBlob(const FsBlobView &blob) {
  // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays)
  auto targetStr = std::make_unique<char[]>(blob.size() + 1); // +1 because of the nullbyte
  blob.read(targetStr.get(), 0, blob.size());
  targetStr[blob.size()] = '\0';
  return targetStr.get();
}

const bf::path &SymlinkBlob::target() const {
  updateAccessTimestamp();
  return _target;

}

}
}
