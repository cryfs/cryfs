#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONDISK_DATA_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONDISK_DATA_H_

#include <cstdlib>
//TODO Move this to a more generic utils
#include "fspp/utils/macros.h"

#include <boost/filesystem/path.hpp>

namespace blobstore {
namespace ondisk {

class Data {
public:
  Data(size_t size);
  virtual ~Data();

  void *data();
  const void *data() const;

  size_t size() const;

  void StoreToFile(const boost::filesystem::path &filepath) const;

private:
  size_t _size;
  void *_data;

  DISALLOW_COPY_AND_ASSIGN(Data);
};

} /* namespace ondisk */
} /* namespace blobstore */

#endif
