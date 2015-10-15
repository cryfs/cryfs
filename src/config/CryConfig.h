#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCONFIG_H_
#define MESSMER_CRYFS_SRC_CONFIG_CRYCONFIG_H_

#include <boost/filesystem/path.hpp>

#include "messmer/cpp-utils/macros.h"

namespace cryfs {

class CryConfig {
public:
  explicit CryConfig(const boost::filesystem::path &configfile);
  virtual ~CryConfig();

  const std::string &RootBlob() const;
  void SetRootBlob(const std::string &value);

  const std::string &EncryptionKey() const;
  void SetEncryptionKey(const std::string &value);

  const std::string &Cipher() const;
  void SetCipher(const std::string &value);

  void save() const;

private:
  boost::filesystem::path _configfile;

  void load();

  std::string _rootBlob;
  std::string _encKey;
  std::string _cipher;

  DISALLOW_COPY_AND_ASSIGN(CryConfig);
};

}

#endif
