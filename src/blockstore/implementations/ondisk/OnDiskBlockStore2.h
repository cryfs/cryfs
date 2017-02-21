#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCKSTORE2_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCKSTORE2_H_

#include "../../interface/BlockStore2.h"
#include <boost/filesystem/path.hpp>
#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/unique_ref.h>
#include "OnDiskBlockStore.h"
#include <cpp-utils/logging/logging.h>

namespace blockstore {
namespace ondisk {

class OnDiskBlockStore2 final: public BlockStore2 {
public:
  explicit OnDiskBlockStore2(const boost::filesystem::path& path)
    : _rootDir(path) {}

  boost::future<bool> tryCreate(const Key &key, const cpputils::Data &data) override {
    auto filepath = _getFilepath(key);
    if (boost::filesystem::exists(filepath)) {
      return boost::make_ready_future(false);
    }

    store(key, data).wait();
    return boost::make_ready_future(true);
  }

  boost::future<bool> remove(const Key &key) override {
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

  boost::future<boost::optional<cpputils::Data>> load(const Key &key) const override {
    auto fileContent = cpputils::Data::LoadFromFile(_getFilepath(key));
    if (fileContent == boost::none) {
      return boost::make_ready_future(boost::optional<cpputils::Data>(boost::none));
    }
    return boost::make_ready_future(boost::optional<cpputils::Data>(_checkAndRemoveHeader(std::move(*fileContent))));
  }

  boost::future<void> store(const Key &key, const cpputils::Data &data) override {
    cpputils::Data fileContent(formatVersionHeaderSize() + data.size());
    std::memcpy(fileContent.data(), FORMAT_VERSION_HEADER.c_str(), formatVersionHeaderSize());
    std::memcpy(fileContent.dataOffset(formatVersionHeaderSize()), data.data(), data.size());
    auto filepath = _getFilepath(key);
    boost::filesystem::create_directory(filepath.parent_path()); // TODO Instead create all of them once at fs creation time?
    fileContent.StoreToFile(filepath);
    return boost::make_ready_future();
  }

private:
  boost::filesystem::path _rootDir;

  static const std::string FORMAT_VERSION_HEADER_PREFIX;
  static const std::string FORMAT_VERSION_HEADER;

  boost::filesystem::path _getFilepath(const Key &key) const {
    std::string keyStr = key.ToString();
    return _rootDir / keyStr.substr(0,3) / keyStr.substr(3);
  }

  static cpputils::Data _checkAndRemoveHeader(const cpputils::Data &data) {
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

  static bool _isAcceptedCryfsHeader(const cpputils::Data &data) {
    return 0 == std::memcmp(data.data(), FORMAT_VERSION_HEADER.c_str(), formatVersionHeaderSize());
  }

  static bool _isOtherCryfsHeader(const cpputils::Data &data) {
    return 0 == std::memcmp(data.data(), FORMAT_VERSION_HEADER_PREFIX.c_str(), FORMAT_VERSION_HEADER_PREFIX.size());
  }

  static unsigned int formatVersionHeaderSize() {
    return FORMAT_VERSION_HEADER.size() + 1; // +1 because of the null byte
  }

  DISALLOW_COPY_AND_ASSIGN(OnDiskBlockStore2);
};

}
}

#endif
