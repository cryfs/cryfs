#include "AtimeOptions.h"

#include <algorithm>
#include <array>

#include <range/v3/view/split.hpp>
#include <range/v3/view/join.hpp>
#include <range/v3/view/filter.hpp>
#include <range/v3/range/conversion.hpp>

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

void extractAllAtimeOptionsAndRemoveOnesUnknownToLibfuse_(string* csv_options, vector<string>* result) {
  *csv_options = ranges::make_subrange(csv_options->begin(), csv_options->end()) | ranges::views::split(',') | ranges::views::filter(
    [&](auto &&elem_) {
        // TODO string_view would be better
        const std::string elem(&*elem_.begin(), ranges::distance(elem_));
        if (is_fuse_unsupported_atime_flag(elem)) {
            result->push_back(elem);
            return false;
        }
        if (is_fuse_supported_atime_flag(elem)) {
            result->push_back(elem);
        }
        return true;
    }) | ranges::views::join(',') | ranges::to<string>();
}
}

vector<string> extractAllAtimeOptionsAndRemoveOnesUnknownToLibfuse(vector<string>* fuseOptions) {
  vector<string> result;
  bool lastOptionWasDashO = false;
  for (size_t i = 0; i < fuseOptions->size(); ++i) {
    string &option = (*fuseOptions)[i];
    if (lastOptionWasDashO) {
      extractAllAtimeOptionsAndRemoveOnesUnknownToLibfuse_(&option, &result);
      if (option.empty()) {
        // All options were removed, remove the empty argument
        fuseOptions->erase(fuseOptions->begin() + i);
        --i;
        // And also remove the now value-less '-o' before it
        fuseOptions->erase(fuseOptions->begin() + i);
        --i;
      }
    }
    lastOptionWasDashO = (option == "-o");
  }

  return result;
}

}
}
