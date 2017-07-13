#include "OnDiskBlockStore2.h"

using std::string;

namespace blockstore {
namespace ondisk {

const string OnDiskBlockStore2::FORMAT_VERSION_HEADER_PREFIX = "cryfs;block;";
const string OnDiskBlockStore2::FORMAT_VERSION_HEADER = OnDiskBlockStore2::FORMAT_VERSION_HEADER_PREFIX + "0";

boost::filesystem::path OnDiskBlockStore2::_getFilepath(const Key &key) const {
  std::string keyStr = key.ToString();
  return _rootDir / keyStr.substr(0,3) / keyStr.substr(3);
}

cpputils::Data OnDiskBlockStore2::_checkAndRemoveHeader(const cpputils::Data &data) {
  if (!_isAcceptedCryfsHeader(data)) {
    if (_isOtherCryfsHeader(data)) {
      throw std::runtime_error("This block is not supported yet. Maybe it was created with a newer version of CryFS?");
    } else {
      throw std::runtime_error("This is not a valid block.");
    }
  }
  cpputils::Data result(data.size() - formatVersionHeaderSize());
  std::memcpy(result.data(), data.dataOffset(formatVersionHeaderSize()), result.size());
  return result;
}

bool OnDiskBlockStore2::_isAcceptedCryfsHeader(const cpputils::Data &data) {
  return 0 == std::memcmp(data.data(), FORMAT_VERSION_HEADER.c_str(), formatVersionHeaderSize());
}

bool OnDiskBlockStore2::_isOtherCryfsHeader(const cpputils::Data &data) {
  return 0 == std::memcmp(data.data(), FORMAT_VERSION_HEADER_PREFIX.c_str(), FORMAT_VERSION_HEADER_PREFIX.size());
}

unsigned int OnDiskBlockStore2::formatVersionHeaderSize() {
  return FORMAT_VERSION_HEADER.size() + 1; // +1 because of the null byte
}

OnDiskBlockStore2::OnDiskBlockStore2(const boost::filesystem::path& path)
    : _rootDir(path) {}

boost::future<bool> OnDiskBlockStore2::tryCreate(const Key &key, const cpputils::Data &data) {
  auto filepath = _getFilepath(key);
  if (boost::filesystem::exists(filepath)) {
    return boost::make_ready_future(false);
  }

  store(key, data).wait();
  return boost::make_ready_future(true);
}

boost::future<bool> OnDiskBlockStore2::remove(const Key &key) {
  auto filepath = _getFilepath(key);
  if (!boost::filesystem::is_regular_file(filepath)) { // TODO Is this branch necessary?
    return boost::make_ready_future(false);
  }
  bool retval = boost::filesystem::remove(filepath);
  if (!retval) {
    cpputils::logging::LOG(cpputils::logging::ERROR, "Couldn't find block {} to remove", key.ToString());
    return boost::make_ready_future(false);
  }
  if (boost::filesystem::is_empty(filepath.parent_path())) {
    boost::filesystem::remove(filepath.parent_path());
  }
  return boost::make_ready_future(true);
}

boost::future<boost::optional<cpputils::Data>> OnDiskBlockStore2::load(const Key &key) const {
  auto fileContent = cpputils::Data::LoadFromFile(_getFilepath(key));
  if (fileContent == boost::none) {
    return boost::make_ready_future(boost::optional<cpputils::Data>(boost::none));
  }
  return boost::make_ready_future(boost::optional<cpputils::Data>(_checkAndRemoveHeader(std::move(*fileContent))));
}

boost::future<void> OnDiskBlockStore2::store(const Key &key, const cpputils::Data &data) {
  cpputils::Data fileContent(formatVersionHeaderSize() + data.size());
  std::memcpy(fileContent.data(), FORMAT_VERSION_HEADER.c_str(), formatVersionHeaderSize());
  std::memcpy(fileContent.dataOffset(formatVersionHeaderSize()), data.data(), data.size());
  auto filepath = _getFilepath(key);
  boost::filesystem::create_directory(filepath.parent_path()); // TODO Instead create all of them once at fs creation time?
  fileContent.StoreToFile(filepath);
  return boost::make_ready_future();
}

uint64_t OnDiskBlockStore2::numBlocks() const {
  uint64_t count = 0;
  for (auto prefixDir = boost::filesystem::directory_iterator(_rootDir); prefixDir != boost::filesystem::directory_iterator(); ++prefixDir) {
    if (boost::filesystem::is_directory(prefixDir->path())) {
      count += std::distance(boost::filesystem::directory_iterator(prefixDir->path()), boost::filesystem::directory_iterator());
    }
  }
  return count;
}

uint64_t OnDiskBlockStore2::estimateNumFreeBytes() const {
  struct statvfs stat;
  ::statvfs(_rootDir.c_str(), &stat);
  return stat.f_bsize*stat.f_bavail;
}

uint64_t OnDiskBlockStore2::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
  if(blockSize <= formatVersionHeaderSize()) {
    return 0;
  }
  return blockSize - formatVersionHeaderSize();
}

void OnDiskBlockStore2::forEachBlock(std::function<void (const Key &)> callback) const {
  for (auto prefixDir = boost::filesystem::directory_iterator(_rootDir); prefixDir != boost::filesystem::directory_iterator(); ++prefixDir) {
    if (boost::filesystem::is_directory(prefixDir->path())) {
      std::string blockKeyPrefix = prefixDir->path().filename().native();
      for (auto block = boost::filesystem::directory_iterator(prefixDir->path()); block != boost::filesystem::directory_iterator(); ++block) {
        std::string blockKeyPostfix = block->path().filename().native();
        callback(Key::FromString(blockKeyPrefix + blockKeyPostfix));
      }
    }
  }
}

}
}
