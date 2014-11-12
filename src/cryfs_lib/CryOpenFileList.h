#ifndef CRYFS_LIB_CRYOPENFILELIST_H_
#define CRYFS_LIB_CRYOPENFILELIST_H_

#include "utils/macros.h"
#include "IdList.h"

namespace cryfs {
class CryOpenFile;
class CryFile;

class CryOpenFileList {
public:
  CryOpenFileList();
  virtual ~CryOpenFileList();

  int open(const CryFile &rhs, int flags);
  CryOpenFile *get(int descriptor);
  void close(int descriptor);

private:
  IdList<CryOpenFile> _open_files;

  DISALLOW_COPY_AND_ASSIGN(CryOpenFileList);
};

}

#endif /* CRYFS_LIB_CRYOPENFILELIST_H_ */
