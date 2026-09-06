#pragma once
#ifndef MESSMER_FSPP_FUSE_MOUNTOPTIONS_H_
#define MESSMER_FSPP_FUSE_MOUNTOPTIONS_H_

#include <functional>
#include <string>
#include <vector>

namespace fspp {
namespace fuse {

// Walks the individual mount options in a fuse argument vector, i.e. the comma separated values
// following each '-o', and calls handleOption() for each of them. So both {..., "-o", "noatime", ...}
// and {..., "-o", "noatime,allow_other", ...} call it once per option.
//
// An option handleOption() returns false for is removed from fuseOptions. A '-o' that is left
// without any value is removed together with it, because libfuse rejects a value-less '-o'.
void forEachMountOption(std::vector<std::string>* fuseOptions, const std::function<bool (const std::string&)>& handleOption);

// Removes the mount options that libfuse 2 accepted on the command line and libfuse 3 doesn't,
// logging a warning for each one. libfuse 3 rejects an option it doesn't know and refuses to mount,
// so without this a user who is used to the libfuse 2 spellings just gets a failed mount.
void removeOptionsRemovedInFuse3(std::vector<std::string>* fuseOptions);

}
}

#endif
