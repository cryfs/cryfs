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
  static constexpr const char* FilesystemFormatVersion = "0.10";

  //TODO No default constructor, pass in config values instead!
  CryConfig();
  CryConfig(CryConfig &&rhs) = default;
  CryConfig(const CryConfig &rhs) = default;

  const std::string &RootBlob() const;
  void SetRootBlob(std::string value);

  const std::string &EncryptionKey() const;
  void SetEncryptionKey(std::string value);

  const std::string &Cipher() const;
  void SetCipher(std::string value);

  const std::string &Version() const;
  void SetVersion(std::string value);

  const std::string &CreatedWithVersion() const;
  void SetCreatedWithVersion(std::string value);

  const std::string &LastOpenedWithVersion() const;
  void SetLastOpenedWithVersion(const std::string &value);

  uint64_t BlocksizeBytes() const;
  void SetBlocksizeBytes(uint64_t value);

  using FilesystemID = cpputils::FixedSizeData<16>;
  const FilesystemID &FilesystemId() const;
  void SetFilesystemId(FilesystemID value);

  // If the exclusive client Id is set, then additional integrity measures (i.e. treating missing blocks as integrity violations) are enabled.
  // Because this only works in a single-client setting, only this one client Id is allowed to access the file system.
  boost::optional<uint32_t> ExclusiveClientId() const;
  void SetExclusiveClientId(boost::optional<uint32_t> value);

  bool missingBlockIsIntegrityViolation() const;

#ifndef CRYFS_NO_COMPATIBILITY
  // This is a trigger to recognize old file systems that didn't have version numbers.
  // Version numbers cannot be disabled, but the file system will be migrated to version numbers automatically.
  bool HasVersionNumbers() const;
  void SetHasVersionNumbers(bool value);

  // This is a trigger to recognize old file systems that didn't have version numbers.
  // Version numbers cannot be disabled, but the file system will be migrated to version numbers automatically.
  bool HasParentPointers() const;
  void SetHasParentPointers(bool value);
#endif

  static CryConfig load(const cpputils::Data &data);
  cpputils::Data save() const;

private:
  std::string _rootBlob;
  std::string _encKey;
  std::string _cipher;
  std::string _version;
  std::string _createdWithVersion;
  std::string _lastOpenedWithVersion;
  uint64_t _blocksizeBytes;
  FilesystemID _filesystemId;
  boost::optional<uint32_t> _exclusiveClientId;
#ifndef CRYFS_NO_COMPATIBILITY
  bool _hasVersionNumbers;
  bool _hasParentPointers;
#endif

  CryConfig &operator=(const CryConfig &rhs) = delete;
};

}

#endif
