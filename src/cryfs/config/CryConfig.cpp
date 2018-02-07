#include "CryConfig.h"

#include <boost/filesystem.hpp>
#include <boost/property_tree/ptree.hpp>
#include <boost/property_tree/json_parser.hpp>
#include <sstream>
#include <gitversion/VersionCompare.h>
#include <cpp-utils/random/Random.h>

namespace bf = boost::filesystem;

using boost::property_tree::ptree;
using boost::optional;
using boost::none;
using std::string;
using std::stringstream;
using cpputils::Data;
using gitversion::VersionCompare;
using cpputils::Random;

namespace cryfs {

constexpr const char* CryConfig::FilesystemFormatVersion;

CryConfig::CryConfig()
: _rootBlob(""), _encKey(""), _cipher(""), _version(""), _createdWithVersion(""), _lastOpenedWithVersion(""),  _blocksizeBytes(0), _filesystemId(FilesystemID::Null()) {
}

CryConfig::CryConfig(CryConfig &&rhs)
: _rootBlob(std::move(rhs._rootBlob)), _encKey(std::move(rhs._encKey)), _cipher(std::move(rhs._cipher)), _version(std::move(rhs._version)), _createdWithVersion(std::move(rhs._createdWithVersion)), _lastOpenedWithVersion(std::move(rhs._lastOpenedWithVersion)), _blocksizeBytes(rhs._blocksizeBytes), _filesystemId(std::move(rhs._filesystemId)) {
}

CryConfig::CryConfig(const CryConfig &rhs)
        : _rootBlob(rhs._rootBlob), _encKey(rhs._encKey), _cipher(rhs._cipher), _version(rhs._version), _createdWithVersion(rhs._createdWithVersion), _lastOpenedWithVersion(rhs._lastOpenedWithVersion), _blocksizeBytes(rhs._blocksizeBytes), _filesystemId(rhs._filesystemId) {
}

CryConfig CryConfig::load(const Data &data) {
  stringstream stream;
  data.StoreToStream(stream);
  ptree pt;
  read_json(stream, pt);

  CryConfig cfg;
  cfg._rootBlob = pt.get<string>("cryfs.rootblob");
  cfg._encKey = pt.get<string>("cryfs.key");
  cfg._cipher = pt.get<string>("cryfs.cipher");
  cfg._version = pt.get<string>("cryfs.version", "0.8"); // CryFS 0.8 didn't specify this field, so if the field doesn't exist, it's 0.8.
  cfg._createdWithVersion = pt.get<string>("cryfs.createdWithVersion", cfg._version); // In CryFS <= 0.9.2, we didn't have this field, but also didn't update cryfs.version, so we can use this field instead.
  cfg._lastOpenedWithVersion = pt.get<string>("cryfs.lastOpenedWithVersion", cfg._version); // In CryFS <= 0.9.8, we didn't have this field, but used the cryfs.version field for this purpose.
  cfg._blocksizeBytes = pt.get<uint64_t>("cryfs.blocksizeBytes", 32832); // CryFS <= 0.9.2 used a 32KB block size which was this physical block size.

  optional<string> filesystemIdOpt = pt.get_optional<string>("cryfs.filesystemId");
  if (filesystemIdOpt == none) {
    cfg._filesystemId = Random::PseudoRandom().getFixedSize<FilesystemID::BINARY_LENGTH>();
  } else {
    cfg._filesystemId = FilesystemID::FromString(*filesystemIdOpt);
  }

  return cfg;
}

Data CryConfig::save() const {
  ptree pt;

  pt.put<string>("cryfs.rootblob", _rootBlob);
  pt.put<string>("cryfs.key", _encKey);
  pt.put<string>("cryfs.cipher", _cipher);
  pt.put<string>("cryfs.version", _version);
  pt.put<string>("cryfs.createdWithVersion", _createdWithVersion);
  pt.put<string>("cryfs.lastOpenedWithVersion", _lastOpenedWithVersion);
  pt.put<uint64_t>("cryfs.blocksizeBytes", _blocksizeBytes);
  pt.put<string>("cryfs.filesystemId", _filesystemId.ToString());

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

const std::string &CryConfig::LastOpenedWithVersion() const {
  return _lastOpenedWithVersion;
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

void CryConfig::SetLastOpenedWithVersion(const std::string &value) {
  _lastOpenedWithVersion = value;
}

uint64_t CryConfig::BlocksizeBytes() const {
  return _blocksizeBytes;
}

void CryConfig::SetBlocksizeBytes(uint64_t value) {
  _blocksizeBytes = value;
}

const CryConfig::FilesystemID &CryConfig::FilesystemId() const {
  return _filesystemId;
}

void CryConfig::SetFilesystemId(const FilesystemID &value) {
  _filesystemId = value;
}

}
