#ifndef CRYFS_LIB_CRYOPENDIRLIST_H_
#define CRYFS_LIB_CRYOPENDIRLIST_H_

#include "utils/macros.h"
#include "IdList.h"

namespace cryfs {
class CryOpenDir;
class CryDir;

class CryOpenDirList {
public:
  CryOpenDirList();
  virtual ~CryOpenDirList();

  int open(const CryDir &rhs);
  CryOpenDir *get(int descriptor);
  void close(int descriptor);

private:
  IdList<CryOpenDir> _open_dirs;

  DISALLOW_COPY_AND_ASSIGN(CryOpenDirList);
};

}

#endif /* CRYFS_LIB_CRYOPENDIRLIST_H_ */
