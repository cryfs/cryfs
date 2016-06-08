#pragma once
#ifndef MESSMER_FSPP_FSINTERFACE_FILE_H_
#define MESSMER_FSPP_FSINTERFACE_FILE_H_

#include "Node.h"
#include <cpp-utils/pointer/unique_ref.h>

namespace fspp {
class Device;
class OpenFile;

class File: public virtual Node {
public:
  virtual ~File() {}

  virtual cpputils::unique_ref<OpenFile> open(int flags) = 0;
  virtual void truncate(off_t size) = 0;
};

}

#endif
