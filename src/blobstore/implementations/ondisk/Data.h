#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONDISK_DATA_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONDISK_DATA_H_

#include <cstdlib>
//TODO Move this to a more generic utils
#include "fspp/utils/macros.h"

namespace blobstore {
namespace ondisk {

class Data {
public:
  Data(size_t size);
  virtual ~Data();

  void *data();
  const void *data() const;

private:
  void *_data;

  DISALLOW_COPY_AND_ASSIGN(Data);
};


} /* namespace ondisk */
} /* namespace blobstore */

#endif
