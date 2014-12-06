#include "OnDiskBlobStore.h"

#include "OnDiskBlob.h"
#include "blobstore/utils/RandomKeyGenerator.h"

using std::unique_ptr;
using std::string;
using std::mutex;
using std::lock_guard;

namespace bf = boost::filesystem;

namespace blobstore {
namespace ondisk {

OnDiskBlobStore::OnDiskBlobStore(const boost::filesystem::path &rootdir)
 : _rootdir(rootdir), _generate_key_mutex() {}

BlobStore::BlobWithKey OnDiskBlobStore::create(size_t size) {
  std::string key = _generateKey();
  auto file_path = _rootdir / key;
  auto blob = OnDiskBlob::CreateOnDisk(file_path, size);

  return BlobStore::BlobWithKey(key, std::move(blob));
}

string OnDiskBlobStore::_generateKey() {
  lock_guard<mutex> lock(_generate_key_mutex);

  string key;
  do {
    key = _generateRandomKey();
  } while (bf::exists(_rootdir / key));

  return key;
}

string OnDiskBlobStore::_generateRandomKey() {
  return RandomKeyGenerator::singleton().create();
}

unique_ptr<Blob> OnDiskBlobStore::load(const string &key) {
  auto file_path = _rootdir / key;
  return OnDiskBlob::LoadFromDisk(file_path);
}

} /* namespace ondisk */
} /* namespace blobstore */
