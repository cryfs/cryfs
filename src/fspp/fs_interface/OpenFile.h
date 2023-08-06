#pragma once
#ifndef MESSMER_FSPP_FSINTERFACE_OPENFILE_H_
#define MESSMER_FSPP_FSINTERFACE_OPENFILE_H_

#include <boost/filesystem.hpp>
#include "Types.h"

namespace fspp {
class Device;

class OpenFile {
public:
  virtual ~OpenFile() {}

  using stat_info = fspp::stat_info;

  virtual fspp::num_bytes_t read(void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) const = 0;
  virtual void write(const void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) = 0;
  virtual void flush() = 0;
  virtual void fsync() = 0;
  virtual void fdatasync() = 0;
};

}

#endif
