#pragma once
#ifndef CRYFS_LIB_CRYCONFIG_H_
#define CRYFS_LIB_CRYCONFIG_H_

#include <boost/filesystem/path.hpp>

#include "messmer/cpp-utils/macros.h"

namespace cryfs {

class CryConfig {
public:
  CryConfig(const boost::filesystem::path &configfile);
  virtual ~CryConfig();

  const std::string &RootBlob() const;
  void SetRootBlob(const std::string &value);

private:
  boost::filesystem::path _configfile;

  void load();
  void save() const;

  std::string _rootBlob;

  DISALLOW_COPY_AND_ASSIGN(CryConfig);
};

}

#endif
