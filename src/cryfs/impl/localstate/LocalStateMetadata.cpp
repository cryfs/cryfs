#include "LocalStateMetadata.h"
#include <boost/property_tree/ptree.hpp>
#include <boost/property_tree/json_parser.hpp>
#include <cpp-utils/random/Random.h>
#include <boost/filesystem.hpp>
#include <blockstore/implementations/integrity/KnownBlockVersions.h>
#include <cryfs/impl/CryfsException.h>

using boost::optional;
using boost::none;
using boost::property_tree::ptree;
using boost::property_tree::write_json;
using boost::property_tree::read_json;
using std::ifstream;
using std::ofstream;
using std::istream;
using std::ostream;
using std::string;
using blockstore::integrity::KnownBlockVersions;
using cpputils::hash::Hash;
using cpputils::Data;
using cpputils::Random;
namespace bf = boost::filesystem;
using namespace cpputils::logging;

namespace cryfs {

LocalStateMetadata::LocalStateMetadata(uint32_t myClientId, Hash encryptionKeyHash)
: _myClientId(myClientId), _encryptionKeyHash(encryptionKeyHash) {}

LocalStateMetadata LocalStateMetadata::loadOrGenerate(const bf::path &statePath, const Data& encryptionKey, bool allowReplacedFilesystem) {
  auto metadataFile = statePath / "metadata";
  auto loaded = load_(metadataFile);
  if (loaded == none) {
    // If it couldn't be loaded, generate a new client id.
    return generate_(metadataFile, encryptionKey);
  }

  if (!allowReplacedFilesystem && loaded->_encryptionKeyHash.digest != cpputils::hash::hash(encryptionKey, loaded->_encryptionKeyHash.salt).digest) {
    throw CryfsException("The filesystem encryption key differs from the last time we loaded this filesystem. Did an attacker replace the file system?", ErrorCode::EncryptionKeyChanged);
  }
  return *loaded;
}

optional<LocalStateMetadata> LocalStateMetadata::load_(const bf::path &metadataFilePath) {
  ifstream file(metadataFilePath.string());
  if (!file.good()) {
    // State file doesn't exist
    return none;
  }
  return deserialize_(file);
}

void LocalStateMetadata::save_(const bf::path &metadataFilePath) const {
  ofstream file(metadataFilePath.string(), std::ios::trunc);
  serialize_(file);
}

namespace {
uint32_t generateClientId_() {
  uint32_t result = 0;
  do {
    result = cpputils::deserialize<uint32_t>(Random::PseudoRandom().getFixedSize<sizeof(uint32_t)>().data());
  } while(result == KnownBlockVersions::CLIENT_ID_FOR_DELETED_BLOCK); // Safety check - CLIENT_ID_FOR_DELETED_BLOCK shouldn't be used by any valid client.
  return result;
}

#ifndef CRYFS_NO_COMPATIBILITY
optional<uint32_t> _tryLoadClientIdFromLegacyFile(const bf::path &metadataFilePath) {
  auto myClientIdFile = metadataFilePath.parent_path() / "myClientId";
  ifstream file(myClientIdFile.string());
  if (!file.good()) {
    return none;
  }

  uint32_t value = 0;
  file >> value;
  file.close();
  bf::remove(myClientIdFile);
  return value;
}
#endif
}

LocalStateMetadata LocalStateMetadata::generate_(const bf::path &metadataFilePath, const Data& encryptionKey) {
  uint32_t myClientId = generateClientId_();
#ifndef CRYFS_NO_COMPATIBILITY
  // In the old format, this was stored in a "myClientId" file. If that file exists, load it from there.
  optional<uint32_t> legacy = _tryLoadClientIdFromLegacyFile(metadataFilePath);
  if (legacy != none) {
    myClientId = *legacy;
  }
#endif

  LocalStateMetadata result(myClientId, cpputils::hash::hash(encryptionKey, cpputils::hash::generateSalt()));
  result.save_(metadataFilePath);
  return result;
}

void LocalStateMetadata::serialize_(ostream& stream) const {
  ptree pt;
  pt.put<uint32_t>("myClientId", myClientId());
  pt.put<string>("encryptionKey.salt", _encryptionKeyHash.salt.ToString());
  pt.put<string>("encryptionKey.hash", _encryptionKeyHash.digest.ToString());

  write_json(stream, pt);
}

LocalStateMetadata LocalStateMetadata::deserialize_(istream& stream) {
	try {
		ptree pt;
		read_json(stream, pt);

		uint32_t myClientId = pt.get<uint32_t>("myClientId");
		string encryptionKeySalt = pt.get<string>("encryptionKey.salt");
		string encryptionKeyDigest = pt.get<string>("encryptionKey.hash");

		return LocalStateMetadata(myClientId, Hash{
			/*.digest = */ cpputils::hash::Digest::FromString(encryptionKeyDigest),
			/*.salt = */ cpputils::hash::Salt::FromString(encryptionKeySalt)
			});
	}
	catch (...) {
		LOG(ERR, "Error loading LocalStateMetadata");
		throw;
	}
}

}
