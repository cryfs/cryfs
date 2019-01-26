#pragma once
#ifndef MESSMER_CRYFS_LOCALSTATE_BASEDIRMETADATA_H_
#define MESSMER_CRYFS_LOCALSTATE_BASEDIRMETADATA_H_

#include <boost/filesystem/path.hpp>
#include <boost/property_tree/ptree.hpp>
#include "../config/CryConfig.h"
#include "LocalStateDir.h"

namespace cryfs {

class BasedirMetadata final {
public:
  static BasedirMetadata load(const LocalStateDir& localStateDir);
  void save();

  BasedirMetadata(const BasedirMetadata&) = delete;
  BasedirMetadata& operator=(const BasedirMetadata&) = delete;
  BasedirMetadata(BasedirMetadata&&) = default;
  BasedirMetadata& operator=(BasedirMetadata&&) = default;

  bool filesystemIdForBasedirIsCorrect(const boost::filesystem::path &basedir, const CryConfig::FilesystemID &filesystemId) const;
  BasedirMetadata& updateFilesystemIdForBasedir(const boost::filesystem::path &basedir, const CryConfig::FilesystemID &filesystemId);

private:
  BasedirMetadata(boost::property_tree::ptree data, boost::filesystem::path filename);

  boost::filesystem::path _filename;
  boost::property_tree::ptree _data;
};

}

#endif
