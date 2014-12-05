#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONDISK_DATA_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONDISK_DATA_H_

#include <cstdlib>
//TODO Move this to a more generic utils
#include "fspp/utils/macros.h"

#include <boost/filesystem/path.hpp>
#include <memory>

namespace blobstore {
namespace ondisk {

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
  static std::unique_ptr<Data> LoadFromFile(const boost::filesystem::path &filepath);

private:
  size_t _size;
  void *_data;

  static size_t _getStreamSize(std::istream &stream);
  void _readFromStream(std::istream &stream);

  DISALLOW_COPY_AND_ASSIGN(Data);
};

} /* namespace ondisk */
} /* namespace blobstore */

#endif
