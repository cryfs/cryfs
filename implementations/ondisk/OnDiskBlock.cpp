#include <cstring>
#include <fstream>
#include <boost/filesystem.hpp>
#include "FileAlreadyExistsException.h"
#include "OnDiskBlock.h"
#include "OnDiskBlockStore.h"
#include "../../utils/FileDoesntExistException.h"

using std::unique_ptr;
using std::make_unique;
using std::istream;
using std::ostream;
using std::ifstream;
using std::ofstream;
using std::ios;

namespace bf = boost::filesystem;

namespace blockstore {
namespace ondisk {

OnDiskBlock::OnDiskBlock(const Key &key, const bf::path &filepath, size_t size)
 : Block(key), _filepath(filepath), _data(size), _dataChanged(false) {
}

OnDiskBlock::OnDiskBlock(const Key &key, const bf::path &filepath, Data &&data)
 : Block(key), _filepath(filepath), _data(std::move(data)), _dataChanged(false) {
}

OnDiskBlock::~OnDiskBlock() {
  flush();
}

const void *OnDiskBlock::data() const {
  return _data.data();
}

void OnDiskBlock::write(const void *source, uint64_t offset, uint64_t size) {
  assert(offset <= _data.size() && offset + size <= _data.size()); //Also check offset < _data->size() because of possible overflow in the addition
  std::memcpy((uint8_t*)_data.data()+offset, source, size);
  _dataChanged = true;
}

size_t OnDiskBlock::size() const {
  return _data.size();
}

unique_ptr<OnDiskBlock> OnDiskBlock::LoadFromDisk(const bf::path &rootdir, const Key &key) {
  auto filepath = rootdir / key.ToString();
  try {
    //If it isn't a file, Data::LoadFromFile() would usually also crash. We still need this extra check
    //upfront, because Data::LoadFromFile() doesn't crash if we give it the path of a directory
    //instead the path of a file.
    if(!bf::is_regular_file(filepath)) {
      return nullptr;
    }
    Data data = Data::LoadFromFile(filepath);
    return unique_ptr<OnDiskBlock>(new OnDiskBlock(key, filepath, std::move(data)));
  } catch (const FileDoesntExistException &e) {
    return nullptr;
  }
}

unique_ptr<OnDiskBlock> OnDiskBlock::CreateOnDisk(const bf::path &rootdir, const Key &key, size_t size) {
  auto filepath = rootdir / key.ToString();
  if (bf::exists(filepath)) {
    return nullptr;
  }

  auto block = unique_ptr<OnDiskBlock>(new OnDiskBlock(key, filepath, size));
  block->_fillDataWithZeroes();
  block->_storeToDisk();
  return block;
}

void OnDiskBlock::RemoveFromDisk(const bf::path &rootdir, const Key &key) {
  auto filepath = rootdir / key.ToString();
  assert(bf::is_regular_file(filepath));
  bf::remove(filepath);
}

void OnDiskBlock::_fillDataWithZeroes() {
  _data.FillWithZeroes();
}

void OnDiskBlock::_storeToDisk() const {
  _data.StoreToFile(_filepath);
}

void OnDiskBlock::flush() {
  if (_dataChanged) {
    _storeToDisk();
    _dataChanged = false;
  }
}

}
}
