#include "CryDir.h"

#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <dirent.h>

#include "messmer/fspp/fuse/FuseErrnoException.h"
#include "CryDevice.h"
#include "CryFile.h"
#include "CryOpenFile.h"

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

namespace bf = boost::filesystem;

using std::unique_ptr;
using std::make_unique;
using std::string;
using std::vector;

using blockstore::Key;

namespace cryfs {

CryDir::CryDir(CryDevice *device, unique_ptr<DirBlob> parent, const Key &key)
: _device(device), _parent(std::move(parent)), _key(key) {
}

CryDir::~CryDir() {
}

void CryDir::stat(struct ::stat *result) const {
  result->st_mode = S_IFDIR | S_IRUSR | S_IXUSR | S_IWUSR;
  return;
  throw FuseErrnoException(ENOTSUP);
}

unique_ptr<fspp::OpenFile> CryDir::createAndOpenFile(const string &name, mode_t mode) {
  auto blob = LoadBlob();
  auto child = _device->CreateBlob();
  Key childkey = child->key();
  blob->AddChildFile(name, childkey);
  //TODO Do we need a return value in createDir for fspp? If not, change fspp Dir interface!
  auto childblob = FileBlob::InitializeEmptyFile(std::move(child));
  return make_unique<CryOpenFile>(std::move(childblob));
}

void CryDir::createDir(const string &name, mode_t mode) {
  auto blob = LoadBlob();
  auto child = _device->CreateBlob();
  Key childkey = child->key();
  blob->AddChildDir(name, childkey);
  DirBlob::InitializeEmptyDir(std::move(child));
}

unique_ptr<DirBlob> CryDir::LoadBlob() const {
  return make_unique<DirBlob>(_device->LoadBlob(_key));
}

void CryDir::rmdir() {
  _parent->RemoveChild(_key);
  _device->RemoveBlob(_key);
}

unique_ptr<vector<fspp::Dir::Entry>> CryDir::children() const {
  auto children = make_unique<vector<fspp::Dir::Entry>>();
  children->push_back(fspp::Dir::Entry(fspp::Dir::EntryType::DIR, "."));
  children->push_back(fspp::Dir::Entry(fspp::Dir::EntryType::DIR, ".."));
  LoadBlob()->AppendChildrenTo(children.get());
  return children;
}

}
