#pragma once
#ifndef FUSEPP_FUSEOPENFILE_H_
#define FUSEPP_FUSEOPENFILE_H_

#include <boost/filesystem.hpp>
#include <sys/stat.h>

#include "utils/macros.h"

namespace fusepp {
class FuseDevice;

namespace bf = boost::filesystem;

class FuseOpenFile {
public:
  FuseOpenFile(const FuseDevice *device, const bf::path &path, int flags);
  virtual ~FuseOpenFile();

  void stat(struct ::stat *result) const;
  void truncate(off_t size) const;
  int read(void *buf, size_t count, off_t offset);
  void write(const void *buf, size_t count, off_t offset);
  void fsync();
  void fdatasync();
private:
  int _descriptor;

  DISALLOW_COPY_AND_ASSIGN(FuseOpenFile);
};

}

#endif /* FUSEPP_FUSEOPENFILE_H_ */
