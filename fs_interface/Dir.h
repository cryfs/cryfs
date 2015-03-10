#pragma once
#ifndef FSPP_DIR_H_
#define FSPP_DIR_H_

#include "Node.h"
#include <memory>
#include <string>

namespace fspp {
class Device;
class File;

class Dir: public virtual Node {
public:
  virtual ~Dir() {}

  enum class EntryType {
    DIR = 0,
    FILE = 1
  };

  struct Entry {
    Entry(EntryType type_, const std::string &name_): type(type_), name(name_) {}
    EntryType type;
    std::string name;
  };

  virtual std::unique_ptr<File> createFile(const std::string &name, mode_t mode) = 0;
  virtual std::unique_ptr<Dir> createDir(const std::string &name, mode_t mode) = 0;
  virtual void rmdir() = 0;

  //TODO Allow alternative implementation returning only children names without more information
  //virtual std::unique_ptr<std::vector<std::string>> children() const = 0;
  virtual std::unique_ptr<std::vector<Entry>> children() const = 0;
};

} /* namespace fspp */

#endif /* FSPP_DIR_H_ */
