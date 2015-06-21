#pragma once
#ifndef FSPP_DIR_H_
#define FSPP_DIR_H_

#include "Node.h"
#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <string>

namespace fspp {
class Device;
class OpenFile;

class Dir: public virtual Node {
public:
  virtual ~Dir() {}

  enum class EntryType {
    DIR = 0,
    FILE = 1,
    SYMLINK = 2
  };

  struct Entry {
    Entry(EntryType type_, const std::string &name_): type(type_), name(name_) {}
    EntryType type;
    std::string name;
  };

  virtual cpputils::unique_ref<OpenFile> createAndOpenFile(const std::string &name, mode_t mode, uid_t uid, gid_t gid) = 0;
  virtual void createDir(const std::string &name, mode_t mode, uid_t uid, gid_t gid) = 0;
  virtual void createSymlink(const std::string &name, const boost::filesystem::path &target, uid_t uid, gid_t gid) = 0;

  //TODO Allow alternative implementation returning only children names without more information
  //virtual std::unique_ptr<std::vector<std::string>> children() const = 0;
  virtual cpputils::unique_ref<std::vector<Entry>> children() const = 0;
};

}

#endif
