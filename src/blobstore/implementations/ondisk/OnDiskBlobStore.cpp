#include "OnDiskBlobStore.h"

#include "OnDiskBlob.h"

#include <fstream>

using std::unique_ptr;
using std::make_unique;
using std::istream;
using std::ifstream;
using std::ofstream;
using std::ios;

namespace blobstore {
namespace ondisk {

OnDiskBlobStore::OnDiskBlobStore(const boost::filesystem::path &rootdir)
 : _rootdir(rootdir) {}

unique_ptr<Blob> OnDiskBlobStore::create(const std::string &key, size_t size) {
  auto blob = make_unique<OnDiskBlob>(size);
  blob->FillDataWithZeroes();
  _storeBlobData(key, blob.get());
  return std::move(blob);
}

void OnDiskBlobStore::_storeBlobData(const std::string &key, const OnDiskBlob *blob) {
  auto file_path = _rootdir / key;
  ofstream file(file_path.c_str(), ios::binary | ios::trunc);

  blob->StoreDataToStream(file);
}

unique_ptr<Blob> OnDiskBlobStore::load(const std::string &key) {
  auto file_path = _rootdir / key;
  ifstream file(file_path.c_str(), ios::binary);

  return _createBlobFromStream(file);
}

unique_ptr<Blob> OnDiskBlobStore::_createBlobFromStream(istream &stream) {
  size_t size = _getStreamSize(stream);

  auto blob = make_unique<OnDiskBlob>(size);
  blob->LoadDataFromStream(stream);
  return std::move(blob);
}

size_t OnDiskBlobStore::_getStreamSize(istream &stream) {
  auto current_pos = stream.tellg();

  //Retrieve length
  stream.seekg(0, stream.end);
  auto endpos = stream.tellg();

  //Restore old position
  stream.seekg(current_pos, stream.beg);

  return endpos - current_pos;
}

} /* namespace ondisk */
} /* namespace blobstore */
