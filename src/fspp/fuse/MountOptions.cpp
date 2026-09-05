#include "MountOptions.h"

#include <array>
#include <utility>

#include <cpp-utils/logging/logging.h>

using std::string;
using std::vector;
using namespace cpputils::logging;

namespace fspp {
namespace fuse {

namespace {
// Removes the options handleOption() returns false for from a comma separated list of options.
void filterCsvOptions(string* csvOptions, const std::function<bool (const string&)>& handleOption) {
  string remaining;
  size_t pos = 0;
  while (true) {
    const size_t next = csvOptions->find(',', pos);
    const size_t end = (next == string::npos) ? csvOptions->size() : next;
    const string option = csvOptions->substr(pos, end - pos);
    if (handleOption(option)) {
      if (!remaining.empty()) {
        remaining += ",";
      }
      remaining += option;
    }
    if (next == string::npos) {
      break;
    }
    pos = next + 1;
  }
  *csvOptions = std::move(remaining);
}

struct RemovedOption final {
  // The option as libfuse 2 spelled it. A name ending in '=' matches any value, e.g. "max_write=".
  const char* name;
  const char* reason;
};

// Verified against libfuse 3.14.0 and 3.17.2: every one of these makes libfuse print
// "fuse: unknown option(s)" and refuse the mount.
constexpr std::array<RemovedOption, 10> REMOVED_IN_FUSE3 = {{
  {"hard_remove", "libfuse 3 decides this itself and hides unlinked but still open files"},
  {"use_ino", "libfuse 3 always uses the inode numbers the file system reports"},
  {"readdir_ino", "libfuse 3 always uses the inode numbers the file system reports"},
  {"direct_io", "in libfuse 3 only the file system can ask for it, per open file"},
  {"nopath", "in libfuse 3 only the file system can ask for it, and CryFS needs the path"},
  {"intr", "in libfuse 3 only the file system can enable interruptible requests"},
  {"intr_signal=", "in libfuse 3 only the file system can enable interruptible requests"},
  {"big_writes", "libfuse 3 always allows large writes"},
  {"large_read", "libfuse 3 dropped it, it only applied to protocol versions before 7.9"},
  {"max_write=", "libfuse 3 negotiates the maximum write size with the kernel itself"},
}};

const RemovedOption* findRemovedOption(const string& option) {
  for (const auto& removed : REMOVED_IN_FUSE3) {
    const string name(removed.name);
    if (name.back() == '=') {
      if (option.size() > name.size() && 0 == option.compare(0, name.size(), name)) {
        return &removed;
      }
    } else if (option == name) {
      return &removed;
    }
  }
  return nullptr;
}
}

void forEachMountOption(vector<string>* fuseOptions, const std::function<bool (const string&)>& handleOption) {
  bool lastOptionWasDashO = false;
  size_t i = 0;
  while (i < fuseOptions->size()) {
    if (lastOptionWasDashO) {
      // A '-o' is always followed by its value, so i >= 1 here.
      filterCsvOptions(&(*fuseOptions)[i], handleOption);
      if ((*fuseOptions)[i].empty()) {
        // Every option in this argument was removed. Remove the now empty argument together with
        // the '-o' before it, because libfuse rejects a '-o' that has no value.
        fuseOptions->erase(fuseOptions->begin() + i - 1, fuseOptions->begin() + i + 1);
        i -= 1;
        lastOptionWasDashO = false;
        continue;
      }
    }
    lastOptionWasDashO = ((*fuseOptions)[i] == "-o");
    ++i;
  }
}

void removeOptionsRemovedInFuse3(vector<string>* fuseOptions) {
  forEachMountOption(fuseOptions, [] (const string& option) {
    const RemovedOption* removed = findRemovedOption(option);
    if (removed == nullptr) {
      return true;
    }
    LOG(WARN, "Ignoring mount option '{}'. It doesn't exist anymore in libfuse 3: {}.", option, removed->reason);
    return false;
  });
}

}
}
