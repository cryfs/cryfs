#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_ONDISK_DATA_H_
#define BLOCKSTORE_IMPLEMENTATIONS_ONDISK_DATA_H_

#include <cstdlib>
//TODO Move this to a more generic utils
#include "fspp/utils/macros.h"

#include <boost/filesystem/path.hpp>
#include <memory>

namespace blockstore {

class Data {
public:
  Data(size_t size);
  Data(Data &&rhs); // move constructor
  virtual ~Data();

  void *data();
  const void *data() const;

  size_t size() const;

  void FillWithZeroes();

  void StoreToFile(const boost::filesystem::path &filepath) const;
  static Data LoadFromFile(const boost::filesystem::path &filepath);

private:
  size_t _size;
  void *_data;

  static void _assertFileExists(const std::ifstream &file, const boost::filesystem::path &filepath);
  static size_t _getStreamSize(std::istream &stream);
  void _readFromStream(std::istream &stream);

  DISALLOW_COPY_AND_ASSIGN(Data);
};

}

#endif
