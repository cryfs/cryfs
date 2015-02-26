#include "CryConfig.h"

#include <boost/filesystem.hpp>
#include <boost/property_tree/ptree.hpp>
#include <boost/property_tree/json_parser.hpp>

namespace bf = boost::filesystem;

using boost::property_tree::ptree;

namespace cryfs {

CryConfig::CryConfig(const bf::path &configfile)
:_configfile(configfile), _rootBlob("") {
  if (bf::exists(_configfile)) {
    load();
  }
}

void CryConfig::load() {
  ptree pt;
  read_json(_configfile.native(), pt);

  _rootBlob = pt.get("cryfs.rootblob", "");
}

void CryConfig::save() const {
  ptree pt;

  pt.put("cryfs.rootblob", _rootBlob);

  write_json(_configfile.native(), pt);
}

const std::string &CryConfig::RootBlob() const {
  return _rootBlob;
}

void CryConfig::SetRootBlob(const std::string &value) {
  _rootBlob = value;
}

CryConfig::~CryConfig() {
  save();
}

}
