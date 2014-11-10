#include <cryfs_lib/CryOpenFileList.h>
#include "CryFile.h"
#include "CryOpenFile.h"

using namespace cryfs;

CryOpenFileList::CryOpenFileList()
  :_open_files() {
}

CryOpenFileList::~CryOpenFileList() {
}

int CryOpenFileList::open(const CryFile &file, int flags) {
  //TODO Reuse descriptors
  int desc = _open_files.size();
  _open_files[desc] = file.open(flags);
  return desc;
}

CryOpenFile *CryOpenFileList::get(int descriptor) {
  return _open_files.at(descriptor).get();
}

void CryOpenFileList::close(int descriptor) {
  //The destructor of the stored CryFile::OpenFile closes the file
  _open_files.erase(descriptor);
}
