#include <cryfs_lib/CryOpenFileList.h>
#include "CryFile.h"
#include "CryOpenFile.h"

using namespace cryfs;

CryOpenFileList::~CryOpenFileList() {
}

CryOpenFileList::CryOpenFileList()
  :_open_files() {
}

int CryOpenFileList::open(const CryFile &file, int flags) {
  return _open_files.add(file.open(flags));
}

CryOpenFile *CryOpenFileList::get(int descriptor) {
  return _open_files.get(descriptor);
}

void CryOpenFileList::close(int descriptor) {
  //The destructor of the stored CryOpenFile closes the file
  _open_files.remove(descriptor);
}
