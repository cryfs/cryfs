#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_ENTRYHELPER_H_
#define MESSMER_CRYFS_FILESYSTEM_ENTRYHELPER_H_

#include <cpp-utils/macros.h>
#include <fspp/fs_interface/Dir.h>
#include <fspp/fs_interface/Node.h>
#include "cryfs/impl/filesystem/rustfsblobstore/RustDirEntry.h"

namespace cryfs {

fspp::Node::stat_info dirEntryToStatInfo(const fsblobstore::rust::RustDirEntry &direntry, fspp::num_bytes_t size);

}

#endif