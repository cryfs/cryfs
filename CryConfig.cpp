#include "CryConfig.h"

#include <boost/filesystem.hpp>
#include <boost/property_tree/ptree.hpp>
#include <boost/property_tree/json_parser.hpp>

namespace bf = boost::filesystem;

using boost::property_tree::ptree;

namespace cryfs {

CryConfig::CryConfig(const bf::path &configfile)
:_configfile(configfile), _root_block("") {
  if (bf::exists(_configfile)) {
    load();
  }
}

void CryConfig::load() {
  ptree pt;
  read_json(_configfile.native(), pt);

  _root_block = pt.get("cryfs.rootblock", "");
}

void CryConfig::save() const {
  ptree pt;

  pt.put("cryfs.rootblock", _root_block);

  write_json(_configfile.native(), pt);
}

const std::string &CryConfig::RootBlock() const {
  return _root_block;
}

void CryConfig::SetRootBlock(const std::string &value) {
  _root_block = value;
}

CryConfig::~CryConfig() {
  save();
}

}
