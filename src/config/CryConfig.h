#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCONFIG_H_
#define MESSMER_CRYFS_SRC_CONFIG_CRYCONFIG_H_

#include <boost/filesystem/path.hpp>

#include "messmer/cpp-utils/macros.h"
#include <iostream>

namespace cryfs {

class CryConfig final {
public:
  CryConfig();
  CryConfig(CryConfig &&rhs);

  const std::string &RootBlob() const;
  void SetRootBlob(const std::string &value);

  const std::string &EncryptionKey() const;
  void SetEncryptionKey(const std::string &value);

  const std::string &Cipher() const;
  void SetCipher(const std::string &value);

  void load(std::istream &loadSource);
  void save(std::ostream &destination) const;

private:
  std::string _rootBlob;
  std::string _encKey;
  std::string _cipher;

  DISALLOW_COPY_AND_ASSIGN(CryConfig);
};

}

#endif
