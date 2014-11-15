#pragma once
#ifndef FUSEPP_FUSEOPENFILELIST_H_
#define FUSEPP_FUSEOPENFILELIST_H_

#include "utils/macros.h"
#include "IdList.h"

namespace fusepp {
class FuseOpenFile;
class FuseFile;

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

}

#endif /* FUSEPP_FUSEOPENFILELIST_H_ */
