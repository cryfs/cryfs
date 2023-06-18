#include "entry_helper.h"
#include "blobstore/implementations/onblocks/utils/Math.h"

namespace cryfs {
fspp::Node::stat_info dirEntryToStatInfo(const fsblobstore::rust::RustDirEntry &dirEntry, fspp::num_bytes_t size) {
  fspp::Node::stat_info result;

  result.mode = dirEntry.mode();
  result.uid = dirEntry.uid();
  result.gid = dirEntry.gid();
  //TODO If possible without performance loss, then for a directory, st_nlink should return number of dir entries (including "." and "..")
  result.nlink = 1;
  result.size = size;
  result.atime = dirEntry.lastAccessTime();
  result.mtime = dirEntry.lastModificationTime();
  result.ctime = dirEntry.lastMetadataChangeTime();
  //TODO Move ceilDivision to general utils which can be used by cryfs as well
  result.blocks = blobstore::onblocks::utils::ceilDivision(size.value(), static_cast<int64_t>(512));
  return result;
}
}
