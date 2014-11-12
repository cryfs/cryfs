#include <cryfs_lib/CryOpenDirList.h>
#include "CryDir.h"
#include "CryOpenDir.h"

using namespace cryfs;

CryOpenDirList::~CryOpenDirList() {
}

CryOpenDirList::CryOpenDirList()
  :_open_dirs() {
}

int CryOpenDirList::open(const CryDir &dir) {
  return _open_dirs.add(dir.opendir());
}

CryOpenDir *CryOpenDirList::get(int descriptor) {
  return _open_dirs.get(descriptor);
}

void CryOpenDirList::close(int descriptor) {
  //The destructor of the stored CryOpenDir closes the dir
  _open_dirs.remove(descriptor);
}
