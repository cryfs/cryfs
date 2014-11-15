#pragma once
#ifndef FSPP_DIR_H_
#define FSPP_DIR_H_

#include <fspp/fs_interface/Node.h>
#include <memory>
#include <string>

namespace fspp {
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

} /* namespace fspp */

#endif /* FSPP_DIR_H_ */
