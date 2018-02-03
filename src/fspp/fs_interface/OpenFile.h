#pragma once
#ifndef MESSMER_FSPP_FSINTERFACE_OPENFILE_H_
#define MESSMER_FSPP_FSINTERFACE_OPENFILE_H_

#include <boost/filesystem.hpp>
#include <sys/stat.h>

namespace fspp {
class Device;

class OpenFile {
public:
  virtual ~OpenFile() {}

  virtual void stat(struct ::stat *result) const = 0;
  virtual void truncate(off_t size) = 0;
  virtual size_t read(void *buf, size_t count, off_t offset) const = 0;
  virtual void write(const void *buf, size_t count, off_t offset) = 0;
  virtual void flush() = 0;
  virtual void fsync() = 0;
  virtual void fdatasync() = 0;
};

}

#endif
