#include "CryDir.h"

#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <dirent.h>

#include "messmer/fspp/fuse/FuseErrnoException.h"
#include "CryDevice.h"
#include "CryFile.h"
#include "CryOpenFile.h"
#include "fsblobstore/SymlinkBlob.h"

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

namespace bf = boost::filesystem;

using std::string;
using std::vector;

using blockstore::Key;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::dynamic_pointer_move;
using boost::optional;
using boost::none;
using cryfs::fsblobstore::DirBlob;

namespace cryfs {

CryDir::CryDir(CryDevice *device, boost::optional<unique_ref<DirBlob>> parent, const Key &key)
: CryNode(device, std::move(parent), key) {
}

CryDir::~CryDir() {
}

unique_ref<fspp::OpenFile> CryDir::createAndOpenFile(const string &name, mode_t mode, uid_t uid, gid_t gid) {
  auto blob = LoadBlob();
  auto child = device()->CreateFileBlob();
  blob->AddChildFile(name, child->key(), mode, uid, gid);
  return make_unique_ref<CryOpenFile>(std::move(child));
}

void CryDir::createDir(const string &name, mode_t mode, uid_t uid, gid_t gid) {
  auto blob = LoadBlob();
  auto child = device()->CreateDirBlob();
  blob->AddChildDir(name, child->key(), mode, uid, gid);
}

unique_ref<DirBlob> CryDir::LoadBlob() const {
  auto blob = CryNode::LoadBlob();
  auto dir_blob = dynamic_pointer_move<DirBlob>(blob);
  ASSERT(dir_blob != none, "Blob does not store a directory");
  return std::move(*dir_blob);
}

unique_ref<vector<fspp::Dir::Entry>> CryDir::children() const {
  auto children = make_unique_ref<vector<fspp::Dir::Entry>>();
  children->push_back(fspp::Dir::Entry(fspp::Dir::EntryType::DIR, "."));
  children->push_back(fspp::Dir::Entry(fspp::Dir::EntryType::DIR, ".."));
  auto blob = LoadBlob();
  blob->AppendChildrenTo(children.get());
  return children;
}

fspp::Dir::EntryType CryDir::getType() const {
  return fspp::Dir::EntryType::DIR;
}

void CryDir::createSymlink(const string &name, const bf::path &target, uid_t uid, gid_t gid) {
  auto blob = LoadBlob();
  auto child = device()->CreateSymlinkBlob(target);
  blob->AddChildSymlink(name, child->key(), uid, gid);
}

}
