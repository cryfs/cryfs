#include "OnDiskBlobStore.h"

#include "OnDiskBlob.h"
#include "blobstore/utils/RandomKeyGenerator.h"

using std::unique_ptr;
using std::make_unique;
using std::string;
using std::mutex;
using std::lock_guard;

namespace bf = boost::filesystem;

namespace blobstore {
namespace ondisk {

OnDiskBlobStore::OnDiskBlobStore(const boost::filesystem::path &rootdir)
 : _rootdir(rootdir) {}

unique_ptr<BlobWithKey> OnDiskBlobStore::create(const std::string &key, size_t size) {
  auto file_path = _rootdir / key;
  auto blob = OnDiskBlob::CreateOnDisk(file_path, size);

  if (!blob) {
    return nullptr;
  }
  return make_unique<BlobWithKey>(key, std::move(blob));
}

bool OnDiskBlobStore::exists(const std::string &key) {
  auto file_path = _rootdir / key;
  return bf::exists(file_path);
}

unique_ptr<Blob> OnDiskBlobStore::load(const string &key) {
  auto file_path = _rootdir / key;
  return OnDiskBlob::LoadFromDisk(file_path);
}

} /* namespace ondisk */
} /* namespace blobstore */
