#include "CryConfig.h"

#include <boost/filesystem.hpp>
#include <boost/property_tree/ptree.hpp>
#include <boost/property_tree/json_parser.hpp>

namespace bf = boost::filesystem;

using boost::property_tree::ptree;
using std::string;

namespace cryfs {

CryConfig::CryConfig(const bf::path &configfile)
:_configfile(configfile), _rootBlob(""), _encKey("") {
  if (bf::exists(_configfile)) {
    load();
  }
}

void CryConfig::load() {
  ptree pt;
  read_json(_configfile.native(), pt);

  _rootBlob = pt.get("cryfs.rootblob", "");
  _encKey = pt.get("cryfs.key", "");
  _cipher = pt.get("cryfs.cipher", "");
}

void CryConfig::save() const {
  ptree pt;

  pt.put("cryfs.rootblob", _rootBlob);
  pt.put("cryfs.key", _encKey);
  pt.put("cryfs.cipher", _cipher);

  write_json(_configfile.native(), pt);
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

CryConfig::~CryConfig() {
  save();
}

}
