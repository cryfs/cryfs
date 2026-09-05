#include "AtimeOptions.h"
#include "MountOptions.h"

#include <algorithm>
#include <array>

using std::string;
using std::vector;

namespace fspp {
namespace fuse {

namespace {
// Options libfuse still accepts on the command line, so we can hand them on after acting on them
// ourselves. 'atime' used to be in this list, but libfuse removed it from its mount option table
// and, since 3.15.0, no longer silently accepts unknown options - passing it on aborts the mount.
bool is_fuse_supported_atime_flag(const std::string& flag) {
  constexpr std::array<const char*, 1> flags = {"noatime"};
  return flags.end() != std::find(flags.begin(), flags.end(), flag);
}

// Options libfuse does not know. We act on them ourselves and must strip them before handing the
// remaining options to libfuse.
bool is_fuse_unsupported_atime_flag(const std::string& flag) {
  constexpr std::array<const char*, 4> flags = {"atime", "strictatime", "relatime", "nodiratime"};
  return flags.end() != std::find(flags.begin(), flags.end(), flag);
}
}

vector<string> extractAllAtimeOptionsAndRemoveOnesUnknownToLibfuse(vector<string>* fuseOptions) {
  vector<string> result;
  forEachMountOption(fuseOptions, [&] (const string& option) {
    if (is_fuse_unsupported_atime_flag(option)) {
      result.push_back(option);
      return false;
    }
    if (is_fuse_supported_atime_flag(option)) {
      result.push_back(option);
    }
    return true;
  });
  return result;
}

}
}
