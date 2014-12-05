#include "OnDiskBlob.h"

#include "OnDiskBlobStore.h"
#include "blobstore/implementations/ondisk/FileAlreadyExistsException.h"

#include <cstring>
#include <fstream>
#include <boost/filesystem.hpp>

using std::unique_ptr;
using std::make_unique;
using std::istream;
using std::ostream;
using std::ifstream;
using std::ofstream;
using std::ios;

namespace bf = boost::filesystem;

namespace blobstore {
namespace ondisk {

OnDiskBlob::OnDiskBlob(const bf::path &filepath, size_t size)
 : _filepath(filepath), _size(size), _data(size) {
}

OnDiskBlob::~OnDiskBlob() {
}

void *OnDiskBlob::data() {
  return _data.data();
}

const void *OnDiskBlob::data() const {
  return _data.data();
}

size_t OnDiskBlob::size() const {
  return _size;
}

unique_ptr<OnDiskBlob> OnDiskBlob::LoadFromDisk(const bf::path &filepath) {
  ifstream file(filepath.c_str(), ios::binary);
  size_t size = _getStreamSize(file);

  auto blob = make_unique<OnDiskBlob>(filepath, size);
  blob->_loadDataFromStream(file);
  return blob;
}

size_t OnDiskBlob::_getStreamSize(istream &stream) {
  auto current_pos = stream.tellg();

  //Retrieve length
  stream.seekg(0, stream.end);
  auto endpos = stream.tellg();

  //Restore old position
  stream.seekg(current_pos, stream.beg);

  return endpos - current_pos;
}

void OnDiskBlob::_loadDataFromStream(istream &stream) {
  stream.read((char*)_data.data(), _size);
}

unique_ptr<OnDiskBlob> OnDiskBlob::CreateOnDisk(const bf::path &filepath, size_t size) {
  _assertFileDoesntExist(filepath);
  auto blob = make_unique<OnDiskBlob>(filepath, size);
  blob->_fillDataWithZeroes();
  blob->_storeToDisk();
  return std::move(blob);
}

void OnDiskBlob::_assertFileDoesntExist(const bf::path &filepath) {
  if (bf::exists(filepath)) {
    throw FileAlreadyExistsException(filepath);
  }
}

void OnDiskBlob::_fillDataWithZeroes() {
  std::memset(_data.data(), 0, _size);
}

void OnDiskBlob::_storeToDisk() const {
  _data.StoreToFile(_filepath);
}

} /* namespace ondisk */
} /* namespace blobstore */
