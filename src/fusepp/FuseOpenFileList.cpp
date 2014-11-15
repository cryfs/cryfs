#include <fusepp/FuseFile.h>
#include <fusepp/FuseOpenFile.h>
#include <fusepp/FuseOpenFileList.h>

using namespace fusepp;

FuseOpenFileList::~FuseOpenFileList() {
}

FuseOpenFileList::FuseOpenFileList()
  :_open_files() {
}

int FuseOpenFileList::open(const FuseFile &file, int flags) {
  return _open_files.add(file.open(flags));
}

FuseOpenFile *FuseOpenFileList::get(int descriptor) {
  return _open_files.get(descriptor);
}

void FuseOpenFileList::close(int descriptor) {
  //The destructor of the stored FuseOpenFile closes the file
  _open_files.remove(descriptor);
}
