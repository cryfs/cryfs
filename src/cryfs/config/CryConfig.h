#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCONFIG_H_
#define MESSMER_CRYFS_SRC_CONFIG_CRYCONFIG_H_

#include <boost/filesystem/path.hpp>

#include <cpp-utils/data/Data.h>
#include <iostream>
#include <cpp-utils/data/FixedSizeData.h>

namespace cryfs {

class CryConfig final {
public:
  //TODO No default constructor, pass in config values instead!
  CryConfig();
  CryConfig(CryConfig &&rhs);

  const std::string &RootBlob() const;
  void SetRootBlob(const std::string &value);

  const std::string &EncryptionKey() const;
  void SetEncryptionKey(const std::string &value);

  const std::string &Cipher() const;
  void SetCipher(const std::string &value);

  const std::string &Version() const;
  void SetVersion(const std::string &value);

  const std::string &CreatedWithVersion() const;
  void SetCreatedWithVersion(const std::string &value);

  uint64_t BlocksizeBytes() const;
  void SetBlocksizeBytes(uint64_t value);

  using FilesystemID = cpputils::FixedSizeData<16>;
  const FilesystemID &FilesystemId() const;
  void SetFilesystemId(const FilesystemID &value);

  static CryConfig load(const cpputils::Data &data);
  cpputils::Data save() const;

private:
  std::string _rootBlob;
  std::string _encKey;
  std::string _cipher;
  std::string _version;
  std::string _createdWithVersion;
  uint64_t _blocksizeBytes;
  FilesystemID _filesystemId;

  DISALLOW_COPY_AND_ASSIGN(CryConfig);
};

}

#endif
