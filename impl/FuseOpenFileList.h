#pragma once
#ifndef FSPP_IMPL_FUSEOPENFILELIST_H_
#define FSPP_IMPL_FUSEOPENFILELIST_H_

#include "../fs_interface/File.h"
#include "../fs_interface/OpenFile.h"
#include "messmer/cpp-utils/macros.h"
#include "IdList.h"

namespace fspp {

class FuseOpenFileList {
public:
  FuseOpenFileList();
  virtual ~FuseOpenFileList();

  int open(const File &rhs, int flags);
  OpenFile *get(int descriptor);
  void close(int descriptor);

private:
  IdList<OpenFile> _open_files;

  DISALLOW_COPY_AND_ASSIGN(FuseOpenFileList);
};

inline FuseOpenFileList::FuseOpenFileList()
  :_open_files() {
}

inline FuseOpenFileList::~FuseOpenFileList() {
}

inline int FuseOpenFileList::open(const File &file, int flags) {
  return _open_files.add(file.open(flags));
}

inline OpenFile *FuseOpenFileList::get(int descriptor) {
  return _open_files.get(descriptor);
}

inline void FuseOpenFileList::close(int descriptor) {
  //The destructor of the stored FuseOpenFile closes the file
  _open_files.remove(descriptor);
}

}

#endif /* FSPP_IMPL_FUSEOPENFILELIST_H_ */
