#pragma once
#ifndef FSPP_SYMLINK_H_
#define FSPP_SYMLINK_H_

#include "Node.h"
#include <memory>
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
