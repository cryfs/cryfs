#pragma once
#ifndef MESSMER_FSPP_FSINTERFACE_FILE_H_
#define MESSMER_FSPP_FSINTERFACE_FILE_H_

#include <cpp-utils/pointer/unique_ref.h>
#include "Types.h"

namespace fspp {
class Device;
class OpenFile;

class File {
public:
  virtual ~File() {}

  virtual cpputils::unique_ref<OpenFile> open(fspp::openflags_t flags) = 0;
  virtual void truncate(fspp::num_bytes_t size) = 0;
};

}

#endif
