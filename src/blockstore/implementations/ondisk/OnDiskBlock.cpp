#include <cstring>
#include <fstream>
#include <boost/filesystem.hpp>
#include "OnDiskBlock.h"
#include "OnDiskBlockStore.h"
#include "../../utils/FileDoesntExistException.h"
#include <cpp-utils/data/DataUtils.h>
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/logging/logging.h>

using std::istream;
using std::ostream;
using std::ifstream;
using std::ofstream;
using std::ios;
using std::string;
using cpputils::Data;
using cpputils::make_unique_ref;
using cpputils::unique_ref;
using boost::optional;
using boost::none;

namespace bf = boost::filesystem;
using namespace cpputils::logging;

namespace blockstore {
namespace ondisk {

const string OnDiskBlock::FORMAT_VERSION_HEADER_PREFIX = "cryfs;block;";
const string OnDiskBlock::FORMAT_VERSION_HEADER = OnDiskBlock::FORMAT_VERSION_HEADER_PREFIX + "0";

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
  std::memcpy(_data.dataOffset(offset), source, size);
  _dataChanged = true;
}

size_t OnDiskBlock::size() const {
  return _data.size();
}

void OnDiskBlock::resize(size_t newSize) {
  _data = cpputils::DataUtils::resize(std::move(_data), newSize);
  _dataChanged = true;
}

bf::path OnDiskBlock::_getFilepath(const bf::path &rootdir, const Key &key) {
  string keyStr = key.ToString();
  return rootdir / keyStr.substr(0,3) / keyStr.substr(3);
}

optional<unique_ref<OnDiskBlock>> OnDiskBlock::LoadFromDisk(const bf::path &rootdir, const Key &key) {
  auto filepath = _getFilepath(rootdir, key);
  try {
    boost::optional<Data> data = _loadFromDisk(filepath);
    if (data == none) {
      return none;
    }
    return make_unique_ref<OnDiskBlock>(key, filepath, std::move(*data));
  } catch (const FileDoesntExistException &e) {
    return none;
  }
}

optional<unique_ref<OnDiskBlock>> OnDiskBlock::CreateOnDisk(const bf::path &rootdir, const Key &key, Data data) {
  auto filepath = _getFilepath(rootdir, key);
  bf::create_directory(filepath.parent_path());
  if (bf::exists(filepath)) {
    return none;
  }

  auto block = make_unique_ref<OnDiskBlock>(key, filepath, std::move(data));
  block->_storeToDisk();
  return std::move(block);
}

void OnDiskBlock::RemoveFromDisk(const bf::path &rootdir, const Key &key) {
  auto filepath = _getFilepath(rootdir, key);
  ASSERT(bf::is_regular_file(filepath), "Block not found on disk");
  bool retval = bf::remove(filepath);
  if (!retval) {
    LOG(ERROR) << "Couldn't find block " << key.ToString() << " to remove";
  }
  if (bf::is_empty(filepath.parent_path())) {
    bf::remove(filepath.parent_path());
  }
}

void OnDiskBlock::_storeToDisk() const {
  std::ofstream file(_filepath.c_str(), std::ios::binary | std::ios::trunc);
  if (!file.good()) {
    throw std::runtime_error("Could not open file for writing");
  }
  file.write(FORMAT_VERSION_HEADER.c_str(), formatVersionHeaderSize());
  if (!file.good()) {
    throw std::runtime_error("Error writing block header");
  }
  _data.StoreToStream(file);
  if (!file.good()) {
    throw std::runtime_error("Error writing block data");
  }
}

optional<Data> OnDiskBlock::_loadFromDisk(const bf::path &filepath) {
  //If it isn't a file, ifstream::good() would return false. We still need this extra check
  //upfront, because ifstream::good() doesn't crash if we give it the path of a directory
  //instead the path of a file.
  if(!bf::is_regular_file(filepath)) {
    return none;
  }
  ifstream file(filepath.c_str(), ios::binary);
  if (!file.good()) {
    return none;
  }
  _checkHeader(&file);
  Data result = Data::LoadFromStream(file);
  //TODO With newer compilers, "return result;" would be enough
  return boost::optional<Data>(std::move(result));
}

void OnDiskBlock::_checkHeader(istream *str) {
  Data header(formatVersionHeaderSize());
  str->read(reinterpret_cast<char*>(header.data()), formatVersionHeaderSize());
  if (!_isAcceptedCryfsHeader(header)) {
    if (_isOtherCryfsHeader(header)) {
      throw std::runtime_error("This block is not supported yet. Maybe it was created with a newer version of CryFS?");
    } else {
      throw std::runtime_error("This is not a valid block.");
    }
  }
}

bool OnDiskBlock::_isAcceptedCryfsHeader(const Data &data) {
  ASSERT(data.size() == formatVersionHeaderSize(), "We extracted the wrong header size from the block.");
  return 0 == std::memcmp(data.data(), FORMAT_VERSION_HEADER.c_str(), formatVersionHeaderSize());
}

bool OnDiskBlock::_isOtherCryfsHeader(const Data &data) {
  ASSERT(data.size() >= FORMAT_VERSION_HEADER_PREFIX.size(), "We extracted the wrong header size from the block.");
  return 0 == std::memcmp(data.data(), FORMAT_VERSION_HEADER_PREFIX.c_str(), FORMAT_VERSION_HEADER_PREFIX.size());
}

unsigned int OnDiskBlock::formatVersionHeaderSize() {
  return FORMAT_VERSION_HEADER.size() + 1; // +1 because of the null byte
}

void OnDiskBlock::flush() {
  std::unique_lock<std::mutex> lock(_mutex);
  if (_dataChanged) {
    _storeToDisk();
    _dataChanged = false;
  }
}

uint64_t OnDiskBlock::blockSizeFromPhysicalBlockSize(uint64_t blockSize) {
  if(blockSize <= formatVersionHeaderSize()) {
    return 0;
  }
  return blockSize - formatVersionHeaderSize();
}

}
}
