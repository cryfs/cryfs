#pragma once
#ifndef MESSMER_FSPP_FUSE_ATIMEOPTIONS_H_
#define MESSMER_FSPP_FUSE_ATIMEOPTIONS_H_

#include <string>
#include <vector>

namespace fspp {
namespace fuse {

// Return a list of all atime options (e.g. atime, noatime, relatime, strictatime, nodiratime) that occur in the
// fuseOptions input. They must be preceded by a '-o', i.e. {..., '-o', 'noatime', ...} and multiple ones can be
// csv-concatenated, i.e. {..., '-o', 'atime,nodiratime', ...}.
// CryFS implements the atime behaviour itself (see Fuse::_createContext), so the returned options are the ones
// we act on. All of them except 'noatime' are also removed from the input fuseOptions, because libfuse either
// never knew them or has since dropped them, and an option libfuse does not know aborts the mount.
std::vector<std::string> extractAllAtimeOptionsAndRemoveOnesUnknownToLibfuse(std::vector<std::string>* fuseOptions);

}
}

#endif
