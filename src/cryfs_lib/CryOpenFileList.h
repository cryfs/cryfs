#ifndef CRYFS_LIB_CRYOPENFILELIST_H_
#define CRYFS_LIB_CRYOPENFILELIST_H_

#include <map>
#include <memory>
#include "utils/macros.h"

namespace cryfs {
class CryFile;
class CryOpenFile;

class CryOpenFileList {
public:
  CryOpenFileList();
  virtual ~CryOpenFileList();

  int open(const CryFile &rhs, int flags);
  CryOpenFile *get(int descriptor);
  void close(int descriptor);

private:
  std::map<int, std::unique_ptr<CryOpenFile>> _open_files;

  DISALLOW_COPY_AND_ASSIGN(CryOpenFileList);
};

}

#endif /* CRYFS_LIB_CRYOPENFILELIST_H_ */
