#pragma once
#ifndef MESSMER_CRYFS_LOCALSTATE_BASEDIRMETADATA_H_
#define MESSMER_CRYFS_LOCALSTATE_BASEDIRMETADATA_H_

#include <boost/filesystem/path.hpp>
#include "../config/CryConfig.h"

namespace cryfs {

class BasedirMetadata final {
public:
  static bool filesystemIdForBasedirIsCorrect(const boost::filesystem::path &basedir, const CryConfig::FilesystemID &filesystemId);
  static void updateFilesystemIdForBasedir(const boost::filesystem::path &basedir, const CryConfig::FilesystemID &filesystemId);
};

}

#endif
