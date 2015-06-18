#pragma once
#ifndef FSPP_FILE_H_
#define FSPP_FILE_H_

#include "Node.h"
#include <messmer/cpp-utils/unique_ref.h>

namespace fspp {
class Device;
class OpenFile;

class File: public virtual Node {
public:
  virtual ~File() {}

  virtual cpputils::unique_ref<OpenFile> open(int flags) const = 0;
  virtual void truncate(off_t size) const = 0;
};

}

#endif
