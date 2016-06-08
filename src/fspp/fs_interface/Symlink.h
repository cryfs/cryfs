#pragma once
#ifndef MESSMER_FSPP_FSINTERFACE_SYMLINK_H_
#define MESSMER_FSPP_FSINTERFACE_SYMLINK_H_

#include "Node.h"
#include <cpp-utils/pointer/unique_ref.h>
#include <string>

namespace fspp {
class Device;

class Symlink: public virtual Node {
public:
  virtual ~Symlink() {}

  virtual boost::filesystem::path target() = 0;
};

}

#endif
