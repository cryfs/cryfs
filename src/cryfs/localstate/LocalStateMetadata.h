#pragma once
#ifndef MESSMER_CRYFS_LOCALSTATE_LOCALSTATEMETADATA_H_
#define MESSMER_CRYFS_LOCALSTATE_LOCALSTATEMETADATA_H_

#include <boost/filesystem/path.hpp>
#include <boost/optional.hpp>
#include <iostream>

namespace cryfs {

class LocalStateMetadata final {
public:

  static LocalStateMetadata loadOrGenerate(const boost::filesystem::path &statePath);

  uint32_t myClientId() const;

private:
  LocalStateMetadata(uint32_t myClientId);

  static boost::optional<LocalStateMetadata> _load(const boost::filesystem::path &metadataFilePath);
  static LocalStateMetadata _deserialize(std::istream& stream);
  static LocalStateMetadata _generate(const boost::filesystem::path &metadataFilePath);
  void _save(const boost::filesystem::path &metadataFilePath) const;
  void _serialize(std::ostream& stream) const;

  const uint32_t _myClientId;
};

inline uint32_t LocalStateMetadata::myClientId() const {
  return _myClientId;
}

}

#endif
