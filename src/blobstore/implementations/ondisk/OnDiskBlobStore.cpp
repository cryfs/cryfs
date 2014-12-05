#include "OnDiskBlobStore.h"

#include "OnDiskBlob.h"

using std::unique_ptr;

namespace blobstore {
namespace ondisk {

OnDiskBlobStore::OnDiskBlobStore(const boost::filesystem::path &rootdir)
 : _rootdir(rootdir) {}

BlobStore::BlobWithKey OnDiskBlobStore::create(const std::string &key, size_t size) {
  auto file_path = _rootdir / key;
  auto blob = OnDiskBlob::CreateOnDisk(file_path, size);

  return BlobStore::BlobWithKey(key, std::move(blob));
}

unique_ptr<Blob> OnDiskBlobStore::load(const std::string &key) {
  auto file_path = _rootdir / key;
  return OnDiskBlob::LoadFromDisk(file_path);
}

} /* namespace ondisk */
} /* namespace blobstore */
