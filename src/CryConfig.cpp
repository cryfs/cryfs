#include "CryConfig.h"

#include <boost/filesystem.hpp>
#include <boost/property_tree/ptree.hpp>
#include <boost/property_tree/json_parser.hpp>

namespace bf = boost::filesystem;

using boost::property_tree::ptree;
using blockstore::encrypted::EncryptionKey;
using std::string;

namespace cryfs {

CryConfig::CryConfig(const bf::path &configfile)
:_configfile(configfile), _rootBlob(""), _encKey(EncryptionKey::CreateRandom()) {
  if (bf::exists(_configfile)) {
    load();
  }
}

void CryConfig::load() {
  ptree pt;
  read_json(_configfile.native(), pt);

  _rootBlob = pt.get("cryfs.rootblob", "");

  string key = pt.get("cryfs.key", "");
  if (key != "") {
    _encKey = EncryptionKey::FromString(key);
  }
}

void CryConfig::save() const {
  ptree pt;

  pt.put("cryfs.rootblob", _rootBlob);
  pt.put("cryfs.key", _encKey.ToString());

  write_json(_configfile.native(), pt);
}

const std::string &CryConfig::RootBlob() const {
  return _rootBlob;
}

void CryConfig::SetRootBlob(const std::string &value) {
  _rootBlob = value;
}

const blockstore::encrypted::EncryptionKey &CryConfig::EncryptionKey() const {
  return _encKey;
}

CryConfig::~CryConfig() {
  save();
}

}
