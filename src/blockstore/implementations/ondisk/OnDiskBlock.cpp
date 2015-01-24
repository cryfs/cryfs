#include <blockstore/implementations/ondisk/FileAlreadyExistsException.h>
#include <blockstore/implementations/ondisk/OnDiskBlock.h>
#include <blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include <blockstore/utils/FileDoesntExistException.h>
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

namespace blockstore {
namespace ondisk {

OnDiskBlock::OnDiskBlock(const Key &key, const bf::path &filepath, size_t size)
 : Block(key), _filepath(filepath), _data(size) {
}

OnDiskBlock::OnDiskBlock(const Key &key, const bf::path &filepath, Data &&data)
 : Block(key), _filepath(filepath), _data(std::move(data)) {
}

OnDiskBlock::~OnDiskBlock() {
  _storeToDisk();
}

void *OnDiskBlock::data() {
  return _data.data();
}

const void *OnDiskBlock::data() const {
  return _data.data();
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

void OnDiskBlock::_fillDataWithZeroes() {
  _data.FillWithZeroes();
}

void OnDiskBlock::_storeToDisk() const {
  _data.StoreToFile(_filepath);
}

void OnDiskBlock::flush() {
  _storeToDisk();
}

}
}
