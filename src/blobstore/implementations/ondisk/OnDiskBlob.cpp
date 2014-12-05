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
 : _filepath(filepath), _data(size) {
}

OnDiskBlob::OnDiskBlob(const bf::path &filepath, Data &&data)
 : _filepath(filepath), _data(std::move(data)) {
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
  return _data.size();
}

unique_ptr<OnDiskBlob> OnDiskBlob::LoadFromDisk(const bf::path &filepath) {
  auto data = Data::LoadFromFile(filepath);

  return unique_ptr<OnDiskBlob>(new OnDiskBlob(filepath, std::move(*data.get())));
}

unique_ptr<OnDiskBlob> OnDiskBlob::CreateOnDisk(const bf::path &filepath, size_t size) {
  _assertFileDoesntExist(filepath);
  auto blob = unique_ptr<OnDiskBlob>(new OnDiskBlob(filepath, size));
  blob->_fillDataWithZeroes();
  blob->_storeToDisk();
  return blob;
}

void OnDiskBlob::_assertFileDoesntExist(const bf::path &filepath) {
  if (bf::exists(filepath)) {
    throw FileAlreadyExistsException(filepath);
  }
}

void OnDiskBlob::_fillDataWithZeroes() {
  _data.FillWithZeroes();
}

void OnDiskBlob::_storeToDisk() const {
  _data.StoreToFile(_filepath);
}

} /* namespace ondisk */
} /* namespace blobstore */
