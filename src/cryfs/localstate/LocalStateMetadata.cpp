#include "LocalStateMetadata.h"
#include <boost/property_tree/ptree.hpp>
#include <boost/property_tree/json_parser.hpp>
#include <cpp-utils/random/Random.h>
#include <boost/filesystem.hpp>
#include <blockstore/implementations/integrity/KnownBlockVersions.h>

using boost::optional;
using boost::none;
using boost::property_tree::ptree;
using boost::property_tree::write_json;
using boost::property_tree::read_json;
using std::ifstream;
using std::ofstream;
using std::istream;
using std::ostream;
using cpputils::Random;
using blockstore::integrity::KnownBlockVersions;
namespace bf = boost::filesystem;

namespace cryfs {

LocalStateMetadata::LocalStateMetadata(uint32_t myClientId)
: _myClientId(myClientId) {}

LocalStateMetadata LocalStateMetadata::loadOrGenerate(const bf::path &statePath) {
  auto metadataFile = statePath / "metadata";
  auto loaded = _load(metadataFile);
  if (loaded != none) {
      return *loaded;
  }
  // If it couldn't be loaded, generate a new client id.
  return _generate(metadataFile);
}

optional<LocalStateMetadata> LocalStateMetadata::_load(const bf::path &metadataFilePath) {
  ifstream file(metadataFilePath.native());
  if (!file.good()) {
    // State file doesn't exist
    return none;
  }
  return _deserialize(file);
}

void LocalStateMetadata::_save(const bf::path &metadataFilePath) const {
  ofstream file(metadataFilePath.native(), std::ios::trunc);
  _serialize(file);
}

namespace {
uint32_t _generateClientId() {
  uint32_t result;
  do {
    result = *reinterpret_cast<uint32_t*>(Random::PseudoRandom().getFixedSize<sizeof(uint32_t)>().data());
  } while(result == KnownBlockVersions::CLIENT_ID_FOR_DELETED_BLOCK); // Safety check - CLIENT_ID_FOR_DELETED_BLOCK shouldn't be used by any valid client.
  return result;
}

#ifndef CRYFS_NO_COMPATIBILITY
optional<uint32_t> _tryLoadClientIdFromLegacyFile(const bf::path &metadataFilePath) {
  auto myClientIdFile = metadataFilePath.parent_path() / "myClientId";
  ifstream file(myClientIdFile.native());
  if (!file.good()) {
    return none;
  }

  uint32_t value;
  file >> value;
  file.close();
  bf::remove(myClientIdFile);
  return value;
}
#endif
}

LocalStateMetadata LocalStateMetadata::_generate(const bf::path &metadataFilePath) {
  uint32_t myClientId = _generateClientId();
#ifndef CRYFS_NO_COMPATIBILITY
  // In the old format, this was stored in a "myClientId" file. If that file exists, load it from there.
  optional<uint32_t> legacy = _tryLoadClientIdFromLegacyFile(metadataFilePath);
  if (legacy != none) {
    myClientId = *legacy;
  }
#endif

  LocalStateMetadata result(myClientId);
  result._save(metadataFilePath);
  return result;
}

void LocalStateMetadata::_serialize(ostream& stream) const {
  ptree pt;
  pt.put<uint32_t>("myClientId", myClientId());

  write_json(stream, pt);
}

LocalStateMetadata LocalStateMetadata::_deserialize(istream& stream) {
  ptree pt;
  read_json(stream, pt);

  uint32_t myClientId = pt.get<uint32_t>("myClientId");

  return LocalStateMetadata(myClientId);
}


}
