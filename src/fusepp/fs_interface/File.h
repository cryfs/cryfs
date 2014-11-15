#pragma once
#ifndef FUSEPP_FILE_H_
#define FUSEPP_FILE_H_

#include <fusepp/fs_interface/Node.h>
#include <memory>

namespace fusepp {
class Device;
class OpenFile;

class File: public virtual Node {
public:
  virtual ~File() {}

  virtual std::unique_ptr<OpenFile> open(int flags) const = 0;
  virtual void truncate(off_t size) const = 0;
  virtual void unlink() = 0;
};

} /* namespace fusepp */

#endif /* FUSEPP_FILE_H_ */
