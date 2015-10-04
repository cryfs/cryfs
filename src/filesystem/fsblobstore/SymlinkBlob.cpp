#include "SymlinkBlob.h"

#include "MagicNumbers.h"
#include <messmer/blockstore/utils/Key.h>
#include <cassert>

using std::string;
using blobstore::Blob;
using cpputils::unique_ref;
using cpputils::make_unique_ref;

namespace bf = boost::filesystem;

namespace cryfs {
namespace fsblobstore {

SymlinkBlob::SymlinkBlob(unique_ref<Blob> blob)
: FsBlob(std::move(blob)), _target(_readTargetFromBlob(baseBlob())) {
}

unique_ref<SymlinkBlob> SymlinkBlob::InitializeSymlink(unique_ref<Blob> blob, const bf::path &target) {
  string targetStr = target.native();
  blob->resize(1 + targetStr.size());
  unsigned char magicNumber = MagicNumbers::SYMLINK;
  blob->write(&magicNumber, 0, 1);
  blob->write(targetStr.c_str(), 1, targetStr.size());
  return make_unique_ref<SymlinkBlob>(std::move(blob));
}

void SymlinkBlob::_checkMagicNumber(const Blob &blob) {
  unsigned char value;
  blob.read(&value, 0, 1);
  ASSERT(value == MagicNumbers::SYMLINK, "Blob is not a symlink blob");
}

bf::path SymlinkBlob::_readTargetFromBlob(const blobstore::Blob &blob) {
  _checkMagicNumber(blob);
  size_t targetStrSize = blob.size() - 1; // -1 because of the magic number
  char targetStr[targetStrSize + 1]; // +1 because of the nullbyte
  blob.read(targetStr, 1, targetStrSize);
  targetStr[targetStrSize] = '\0';
  return targetStr;
}

const bf::path &SymlinkBlob::target() const {
  return _target;
}

off_t SymlinkBlob::lstat_size() const {
  return target().native().size();
}

}
}
