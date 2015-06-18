#include "CryDir.h"

#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <dirent.h>

#include "messmer/fspp/fuse/FuseErrnoException.h"
#include "CryDevice.h"
#include "CryFile.h"
#include "CryOpenFile.h"
#include "impl/SymlinkBlob.h"

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

namespace bf = boost::filesystem;

using std::unique_ptr;
using std::make_unique;
using std::string;
using std::vector;

using blockstore::Key;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::optional;
using boost::none;

namespace cryfs {

CryDir::CryDir(CryDevice *device, boost::optional<unique_ref<DirBlob>> parent, const Key &key)
: CryNode(device, std::move(parent), key) {
}

CryDir::~CryDir() {
}

unique_ptr<fspp::OpenFile> CryDir::createAndOpenFile(const string &name, mode_t mode, uid_t uid, gid_t gid) {
  auto blob = LoadBlob();
  if (blob == none) {
    //TODO Return correct fuse error
    throw FuseErrnoException(EIO);
  }
  auto child = device()->CreateBlob();
  Key childkey = child->key();
  (*blob)->AddChildFile(name, childkey, mode, uid, gid);
  //TODO Do we need a return value in createDir for fspp? If not, change fspp Dir interface!
  auto childblob = FileBlob::InitializeEmptyFile(std::move(child));
  return make_unique<CryOpenFile>(std::move(childblob));
}

void CryDir::createDir(const string &name, mode_t mode, uid_t uid, gid_t gid) {
  auto blob = LoadBlob();
  if (blob == none) {
    //TODO Return correct fuse error
    throw FuseErrnoException(EIO);
  }
  auto child = device()->CreateBlob();
  Key childkey = child->key();
  (*blob)->AddChildDir(name, childkey, mode, uid, gid);
  DirBlob::InitializeEmptyDir(std::move(child), device());
}

optional<unique_ref<DirBlob>> CryDir::LoadBlob() const {
  auto blob = CryNode::LoadBlob();
  if(blob == none) {
    return none;
  }
  //TODO Without const_cast?
  return make_unique_ref<DirBlob>(std::move(*blob), const_cast<CryDevice*>(device()));
}

unique_ptr<vector<fspp::Dir::Entry>> CryDir::children() const {
  auto children = make_unique<vector<fspp::Dir::Entry>>();
  children->push_back(fspp::Dir::Entry(fspp::Dir::EntryType::DIR, "."));
  children->push_back(fspp::Dir::Entry(fspp::Dir::EntryType::DIR, ".."));
  auto blob = LoadBlob();
  if (blob == none) {
    //TODO Return correct fuse error
    throw FuseErrnoException(EIO);
  }
  (*blob)->AppendChildrenTo(children.get());
  return children;
}

fspp::Dir::EntryType CryDir::getType() const {
  return fspp::Dir::EntryType::DIR;
}

void CryDir::createSymlink(const string &name, const bf::path &target, uid_t uid, gid_t gid) {
  auto blob = LoadBlob();
  if (blob == none) {
    //TODO Return correct fuse error
    throw FuseErrnoException(EIO);
  }
  auto child = device()->CreateBlob();
  Key childkey = child->key();
  (*blob)->AddChildSymlink(name, childkey, uid, gid);
  SymlinkBlob::InitializeSymlink(std::move(child), target);
}

}
