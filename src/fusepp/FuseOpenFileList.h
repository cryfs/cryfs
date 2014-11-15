#pragma once
#ifndef FUSEPP_FUSEOPENFILELIST_H_
#define FUSEPP_FUSEOPENFILELIST_H_

#include "utils/macros.h"
#include "IdList.h"
#include "FuseFile.h"
#include "FuseOpenFile.h"

namespace fusepp {

class FuseOpenFileList {
public:
  FuseOpenFileList();
  virtual ~FuseOpenFileList();

  int open(const FuseFile &rhs, int flags);
  FuseOpenFile *get(int descriptor);
  void close(int descriptor);

private:
  IdList<FuseOpenFile> _open_files;

  DISALLOW_COPY_AND_ASSIGN(FuseOpenFileList);
};

inline FuseOpenFileList::FuseOpenFileList()
  :_open_files() {
}

inline int FuseOpenFileList::open(const FuseFile &file, int flags) {
  return _open_files.add(file.open(flags));
}

inline FuseOpenFile *FuseOpenFileList::get(int descriptor) {
  return _open_files.get(descriptor);
}

inline void FuseOpenFileList::close(int descriptor) {
  //The destructor of the stored FuseOpenFile closes the file
  _open_files.remove(descriptor);
}

}

#endif /* FUSEPP_FUSEOPENFILELIST_H_ */
