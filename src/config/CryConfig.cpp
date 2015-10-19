#include "CryConfig.h"

#include <boost/filesystem.hpp>
#include <boost/property_tree/ptree.hpp>
#include <boost/property_tree/json_parser.hpp>

namespace bf = boost::filesystem;

using boost::property_tree::ptree;
using std::string;
using std::istream;
using std::ostream;

namespace cryfs {

CryConfig::CryConfig()
: _rootBlob(""), _encKey(""), _cipher("") {
}

CryConfig::CryConfig(CryConfig &&rhs)
: _rootBlob(std::move(rhs._rootBlob)), _encKey(std::move(rhs._encKey)), _cipher(std::move(rhs._cipher)) {
}

void CryConfig::load(istream &loadSource) {
  ptree pt;
  read_json(loadSource, pt);

  _rootBlob = pt.get("cryfs.rootblob", "");
  _encKey = pt.get("cryfs.key", "");
  _cipher = pt.get("cryfs.cipher", "");
}

void CryConfig::save(ostream &writeDestination) const {
  ptree pt;

  pt.put("cryfs.rootblob", _rootBlob);
  pt.put("cryfs.key", _encKey);
  pt.put("cryfs.cipher", _cipher);

  write_json(writeDestination, pt);
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

}
