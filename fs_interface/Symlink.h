#pragma once
#ifndef MESSMER_FSPP_FSINTERFACE_SYMLINK_H_
#define MESSMER_FSPP_FSINTERFACE_SYMLINK_H_

#include "Node.h"
#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <string>

namespace fspp {
class Device;

class Symlink: public virtual Node {
public:
  virtual ~Symlink() {}

  virtual boost::filesystem::path target() const = 0;
};

}

#endif
