#pragma once
#ifndef MESSMER_FSPP_IMPL_FUSEOPENFILELIST_H_
#define MESSMER_FSPP_IMPL_FUSEOPENFILELIST_H_

#include "../fs_interface/File.h"
#include "../fs_interface/OpenFile.h"
#include <cpp-utils/macros.h>
#include "IdList.h"

namespace fspp {

class FuseOpenFileList final {
public:
  FuseOpenFileList();
  ~FuseOpenFileList();

  int open(cpputils::unique_ref<OpenFile> file);
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

inline int FuseOpenFileList::open(cpputils::unique_ref<OpenFile> file) {
  return _open_files.add(std::move(file));
}

inline OpenFile *FuseOpenFileList::get(int descriptor) {
  return _open_files.get(descriptor);
}

inline void FuseOpenFileList::close(int descriptor) {
  //The destructor of the stored FuseOpenFile closes the file
  _open_files.remove(descriptor);
}

}

#endif
