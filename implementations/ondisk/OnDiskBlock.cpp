#include <cstring>
#include <fstream>
#include <boost/filesystem.hpp>
#include "OnDiskBlock.h"
#include "OnDiskBlockStore.h"
#include "../../utils/FileDoesntExistException.h"
#include <messmer/cpp-utils/assert/assert.h>

using std::istream;
using std::ostream;
using std::ifstream;
using std::ofstream;
using std::ios;
using cpputils::Data;
using cpputils::make_unique_ref;
using cpputils::unique_ref;
using boost::optional;
using boost::none;

namespace bf = boost::filesystem;

namespace blockstore {
namespace ondisk {

OnDiskBlock::OnDiskBlock(const Key &key, const bf::path &filepath, Data data)
 : Block(key), _filepath(filepath), _data(std::move(data)), _dataChanged(false), _mutex() {
}

OnDiskBlock::~OnDiskBlock() {
  flush();
}

const void *OnDiskBlock::data() const {
  return _data.data();
}

void OnDiskBlock::write(const void *source, uint64_t offset, uint64_t size) {
  ASSERT(offset <= _data.size() && offset + size <= _data.size(), "Write outside of valid area"); //Also check offset < _data->size() because of possible overflow in the addition
  std::memcpy((uint8_t*)_data.data()+offset, source, size);
  _dataChanged = true;
}

size_t OnDiskBlock::size() const {
  return _data.size();
}

optional<unique_ref<OnDiskBlock>> OnDiskBlock::LoadFromDisk(const bf::path &rootdir, const Key &key) {
  auto filepath = rootdir / key.ToString();
  try {
    //If it isn't a file, Data::LoadFromFile() would usually also crash. We still need this extra check
    //upfront, because Data::LoadFromFile() doesn't crash if we give it the path of a directory
    //instead the path of a file.
    //TODO Data::LoadFromFile now returns boost::optional. Do we then still need this?
    if(!bf::is_regular_file(filepath)) {
      return none;
    }
    boost::optional<Data> data = Data::LoadFromFile(filepath);
    if (!data) {
      return none;
    }
    return make_unique_ref<OnDiskBlock>(key, filepath, std::move(*data));
  } catch (const FileDoesntExistException &e) {
    return none;
  }
}

optional<unique_ref<OnDiskBlock>> OnDiskBlock::CreateOnDisk(const bf::path &rootdir, const Key &key, Data data) {
  auto filepath = rootdir / key.ToString();
  if (bf::exists(filepath)) {
    return none;
  }

  auto block = make_unique_ref<OnDiskBlock>(key, filepath, std::move(data));
  block->_storeToDisk();
  return std::move(block);
}

void OnDiskBlock::RemoveFromDisk(const bf::path &rootdir, const Key &key) {
  auto filepath = rootdir / key.ToString();
  ASSERT(bf::is_regular_file(filepath), "Block not found on disk");
  bf::remove(filepath);
}

void OnDiskBlock::_fillDataWithZeroes() {
  _data.FillWithZeroes();
  _dataChanged = true;
}

void OnDiskBlock::_storeToDisk() const {
  _data.StoreToFile(_filepath);
}

void OnDiskBlock::flush() {
  std::unique_lock<std::mutex> lock(_mutex);
  if (_dataChanged) {
    _storeToDisk();
    _dataChanged = false;
  }
}

}
}
