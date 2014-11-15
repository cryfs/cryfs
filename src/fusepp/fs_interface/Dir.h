#pragma once
#ifndef FUSEPP_DIR_H_
#define FUSEPP_DIR_H_

#include <fusepp/fs_interface/Node.h>
#include <memory>
#include <string>

namespace fusepp {
class Device;
class File;

class Dir: public virtual Node {
public:
  virtual ~Dir() {}

  virtual std::unique_ptr<File> createFile(const std::string &name, mode_t mode) = 0;
  virtual std::unique_ptr<Dir> createDir(const std::string &name, mode_t mode) = 0;
  virtual void rmdir() = 0;

  virtual std::unique_ptr<std::vector<std::string>> children() const = 0;
};

} /* namespace fusepp */

#endif /* FUSEPP_DIR_H_ */
