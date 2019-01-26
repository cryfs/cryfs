#pragma once
#ifndef MESSMER_CRYFS_LOCALSTATE_LOCALSTATEMETADATA_H_
#define MESSMER_CRYFS_LOCALSTATE_LOCALSTATEMETADATA_H_

#include <boost/filesystem/path.hpp>
#include <boost/optional.hpp>
#include <iostream>
#include <cpp-utils/crypto/hash/Hash.h>

namespace cryfs {

class LocalStateMetadata final {
public:

  static LocalStateMetadata loadOrGenerate(const boost::filesystem::path &statePath, const cpputils::Data& encryptionKey, bool allowReplacedFilesystem);

  uint32_t myClientId() const;

private:
  const uint32_t _myClientId;
  const cpputils::hash::Hash _encryptionKeyHash;

  static boost::optional<LocalStateMetadata> _load(const boost::filesystem::path &metadataFilePath);
  static LocalStateMetadata _deserialize(std::istream& stream);
  static LocalStateMetadata _generate(const boost::filesystem::path &metadataFilePath, const cpputils::Data& encryptionKey);
  void _save(const boost::filesystem::path &metadataFilePath) const;
  void _serialize(std::ostream& stream) const;

  LocalStateMetadata(uint32_t myClientId, cpputils::hash::Hash encryptionKey);
};

inline uint32_t LocalStateMetadata::myClientId() const {
  return _myClientId;
}

}

#endif
