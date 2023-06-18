#pragma once
#ifndef MESSMER_FSPP_FSINTERFACE_SYMLINK_H_
#define MESSMER_FSPP_FSINTERFACE_SYMLINK_H_

#include <cpp-utils/pointer/unique_ref.h>
#include <string>
#include <boost/filesystem/path.hpp>

namespace fspp {
class Device;

class Symlink {
public:
  virtual ~Symlink() {}

  virtual boost::filesystem::path target() = 0;
};

}

#endif
