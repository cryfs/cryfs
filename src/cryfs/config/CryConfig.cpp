#include "CryConfig.h"

#include <boost/filesystem.hpp>
#include <boost/property_tree/ptree.hpp>
#include <boost/property_tree/json_parser.hpp>
#include <sstream>
#include <gitversion/VersionCompare.h>

namespace bf = boost::filesystem;

using boost::property_tree::ptree;
using std::string;
using std::stringstream;
using cpputils::Data;
using gitversion::VersionCompare;

namespace cryfs {

CryConfig::CryConfig()
: _rootBlob(""), _encKey(""), _cipher(""), _version(""), _createdWithVersion(""), _blocksizeBytes(0) {
}

CryConfig::CryConfig(CryConfig &&rhs)
: _rootBlob(std::move(rhs._rootBlob)), _encKey(std::move(rhs._encKey)), _cipher(std::move(rhs._cipher)), _version(std::move(rhs._version)), _createdWithVersion(std::move(rhs._createdWithVersion)), _blocksizeBytes(rhs._blocksizeBytes) {
}

CryConfig CryConfig::load(const Data &data) {
  stringstream stream;
  data.StoreToStream(stream);
  ptree pt;
  read_json(stream, pt);

  CryConfig cfg;
  cfg._rootBlob = pt.get("cryfs.rootblob", "");
  cfg._encKey = pt.get("cryfs.key", "");
  cfg._cipher = pt.get("cryfs.cipher", "");
  cfg._version = pt.get("cryfs.version", "0.8"); // CryFS 0.8 didn't specify this field, so if the field doesn't exist, it's 0.8.
  cfg._createdWithVersion = pt.get("cryfs.createdWithVersion", cfg._version); // In Crfys <= 0.9.2, we didn't have this field, but also didn't update cryfs.version, so we can use this field instead.
  cfg._blocksizeBytes = pt.get<uint64_t>("cryfs.blocksizeBytes", 32832); // CryFS <= 0.9.2 used a 32KB block size which was this physical block size.
  return cfg;
}

Data CryConfig::save() const {
  ptree pt;

  pt.put("cryfs.rootblob", _rootBlob);
  pt.put("cryfs.key", _encKey);
  pt.put("cryfs.cipher", _cipher);
  pt.put("cryfs.version", _version);
  pt.put("cryfs.createdWithVersion", _createdWithVersion);
  pt.put<uint64_t>("cryfs.blocksizeBytes", _blocksizeBytes);

  stringstream stream;
  write_json(stream, pt);
  return Data::LoadFromStream(stream);
}

const std::string &CryConfig::RootBlob() const {
  return _rootBlob;
}

void CryConfig::SetRootBlob(const std::string &value) {
  _rootBlob = value;
}

const string &CryConfig::EncryptionKey() const {
  return _encKey;
}

void CryConfig::SetEncryptionKey(const std::string &value) {
  _encKey = value;
}

const std::string &CryConfig::Cipher() const {
  return _cipher;
};

void CryConfig::SetCipher(const std::string &value) {
  _cipher = value;
}

const std::string &CryConfig::Version() const {
  return _version;
}

void CryConfig::SetVersion(const std::string &value) {
  _version = value;
}

const std::string &CryConfig::CreatedWithVersion() const {
  return _createdWithVersion;
}

void CryConfig::SetCreatedWithVersion(const std::string &value) {
  _createdWithVersion = value;
}

uint64_t CryConfig::BlocksizeBytes() const {
  return _blocksizeBytes;
}

void CryConfig::SetBlocksizeBytes(uint64_t value) {
  _blocksizeBytes = value;
}

}
