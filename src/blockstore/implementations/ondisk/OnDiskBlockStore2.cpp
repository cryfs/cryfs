#include "OnDiskBlockStore2.h"
#include <boost/filesystem.hpp>
#include <cpp-utils/system/diskspace.h>

using std::string;
using boost::optional;
using boost::none;
using cpputils::Data;

namespace blockstore {
namespace ondisk {

const string OnDiskBlockStore2::FORMAT_VERSION_HEADER_PREFIX = "cryfs;block;";
const string OnDiskBlockStore2::FORMAT_VERSION_HEADER = OnDiskBlockStore2::FORMAT_VERSION_HEADER_PREFIX + "0";
namespace {
constexpr size_t PREFIX_LENGTH = 3;
constexpr size_t POSTFIX_LENGTH = BlockId::STRING_LENGTH - PREFIX_LENGTH;
constexpr const char* ALLOWED_BLOCKID_CHARACTERS = "0123456789ABCDEF";
}

boost::filesystem::path OnDiskBlockStore2::_getFilepath(const BlockId &blockId) const {
  const std::string blockIdStr = blockId.ToString();
  return _rootDir / blockIdStr.substr(0, PREFIX_LENGTH) / blockIdStr.substr(PREFIX_LENGTH);
}

Data OnDiskBlockStore2::_checkAndRemoveHeader(const Data &data) {
  if (!_isAcceptedCryfsHeader(data)) {
    if (_isOtherCryfsHeader(data)) {
      throw std::runtime_error("This block is not supported yet. Maybe it was created with a newer version of CryFS?");
    } else {
      throw std::runtime_error("This is not a valid block.");
    }
  }
  Data result(data.size() - formatVersionHeaderSize());
  std::memcpy(result.data(), data.dataOffset(formatVersionHeaderSize()), result.size());
  return result;
}

bool OnDiskBlockStore2::_isAcceptedCryfsHeader(const Data &data) {
  return 0 == std::memcmp(data.data(), FORMAT_VERSION_HEADER.c_str(), formatVersionHeaderSize());
}

bool OnDiskBlockStore2::_isOtherCryfsHeader(const Data &data) {
  return 0 == std::memcmp(data.data(), FORMAT_VERSION_HEADER_PREFIX.c_str(), FORMAT_VERSION_HEADER_PREFIX.size());
}

unsigned int OnDiskBlockStore2::formatVersionHeaderSize() {
  return FORMAT_VERSION_HEADER.size() + 1; // +1 because of the null byte
}

OnDiskBlockStore2::OnDiskBlockStore2(const boost::filesystem::path& path)
    : _rootDir(path) {}

bool OnDiskBlockStore2::tryCreate(const BlockId &blockId, const Data &data) {
  auto filepath = _getFilepath(blockId);
  if (boost::filesystem::exists(filepath)) {
    return false;
  }

  store(blockId, data);
  return true;
}

bool OnDiskBlockStore2::remove(const BlockId &blockId) {
  auto filepath = _getFilepath(blockId);
  if (!boost::filesystem::is_regular_file(filepath)) { // TODO Is this branch necessary?
    return false;
  }
  const bool retval = boost::filesystem::remove(filepath);
  if (!retval) {
    cpputils::logging::LOG(cpputils::logging::ERR, "Couldn't find block {} to remove", blockId.ToString());
    return false;
  }
  if (boost::filesystem::is_empty(filepath.parent_path())) {
    boost::filesystem::remove(filepath.parent_path());
  }
  return true;
}

optional<Data> OnDiskBlockStore2::load(const BlockId &blockId) const {
  auto fileContent = Data::LoadFromFile(_getFilepath(blockId));
  if (fileContent == none) {
    return boost::none;
  }
  return _checkAndRemoveHeader(*fileContent);
}

void OnDiskBlockStore2::store(const BlockId &blockId, const Data &data) {
  Data fileContent(formatVersionHeaderSize() + data.size());
  std::memcpy(fileContent.data(), FORMAT_VERSION_HEADER.c_str(), formatVersionHeaderSize());
  std::memcpy(fileContent.dataOffset(formatVersionHeaderSize()), data.data(), data.size());
  auto filepath = _getFilepath(blockId);
  boost::filesystem::create_directory(filepath.parent_path()); // TODO Instead create all of them once at fs creation time?
  fileContent.StoreToFile(filepath);
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
	return cpputils::free_disk_space_in_bytes(_rootDir);
}

uint64_t OnDiskBlockStore2::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
  if(blockSize <= formatVersionHeaderSize()) {
    return 0;
  }
  return blockSize - formatVersionHeaderSize();
}

void OnDiskBlockStore2::forEachBlock(std::function<void (const BlockId &)> callback) const {
  for (auto prefixDir = boost::filesystem::directory_iterator(_rootDir); prefixDir != boost::filesystem::directory_iterator(); ++prefixDir) {
    if (!boost::filesystem::is_directory(prefixDir->path())) {
      continue;
    }

    const std::string blockIdPrefix = prefixDir->path().filename().string();
    if (blockIdPrefix.size() != PREFIX_LENGTH || std::string::npos != blockIdPrefix.find_first_not_of(ALLOWED_BLOCKID_CHARACTERS)) {
      // directory has wrong length or an invalid character
      continue;
    }

    for (auto block = boost::filesystem::directory_iterator(prefixDir->path()); block != boost::filesystem::directory_iterator(); ++block) {
      const std::string blockIdPostfix = block->path().filename().string();
      if (blockIdPostfix.size() != POSTFIX_LENGTH || std::string::npos != blockIdPostfix.find_first_not_of(ALLOWED_BLOCKID_CHARACTERS)) {
        // filename has wrong length or an invalid character
        continue;
      }

      callback(BlockId::FromString(blockIdPrefix + blockIdPostfix));
    }
  }
}

}
}
