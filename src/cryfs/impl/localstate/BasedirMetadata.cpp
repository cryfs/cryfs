#include "BasedirMetadata.h"
#include <boost/property_tree/ptree.hpp>
#include <boost/property_tree/json_parser.hpp>
#include <vendor_cryptopp/sha.h>
#include <boost/filesystem/operations.hpp>
#include "LocalStateDir.h"
#include <cpp-utils/logging/logging.h>

namespace bf = boost::filesystem;
using boost::property_tree::ptree;
using boost::property_tree::write_json;
using boost::property_tree::read_json;
using boost::none;
using std::ostream;
using std::istream;
using std::ifstream;
using std::ofstream;
using std::string;
using namespace cpputils::logging;

namespace cryfs {

namespace {

ptree _load(const bf::path &metadataFilePath) {
	try {
		ptree result;

		ifstream file(metadataFilePath.string());
		if (file.good()) {
			read_json(file, result);
		}

		return result;
	}
	catch (...) {
		LOG(ERR, "Error loading BasedirMetadata");
		throw;
	}
}

void _save(const bf::path &metadataFilePath, const ptree& data) {
  ofstream file(metadataFilePath.string(), std::ios::trunc);
  write_json(file, data);
}

string jsonPathForBasedir(const bf::path &basedir) {
  return bf::canonical(basedir).string() + ".filesystemId";
}

}

BasedirMetadata::BasedirMetadata(ptree data, bf::path filename)
  :_filename(std::move(filename)), _data(std::move(data)) {}

BasedirMetadata BasedirMetadata::load(const LocalStateDir& localStateDir) {
  auto filename = localStateDir.forBasedirMetadata();
  auto loaded = _load(filename);
  return BasedirMetadata(std::move(loaded), std::move(filename));
}

void BasedirMetadata::save() {
  _save(_filename, _data);
}

bool BasedirMetadata::filesystemIdForBasedirIsCorrect(const bf::path &basedir, const CryConfig::FilesystemID &filesystemId) const {
  auto entry = _data.get_optional<string>(jsonPathForBasedir(basedir));
  if (entry == boost::none) {
    return true; // Basedir not known in local state yet.
  }
  auto filesystemIdFromState = CryConfig::FilesystemID::FromString(*entry);
  return filesystemIdFromState == filesystemId;
}

BasedirMetadata& BasedirMetadata::updateFilesystemIdForBasedir(const bf::path &basedir, const CryConfig::FilesystemID &filesystemId) {
  _data.put<string>(jsonPathForBasedir(basedir), filesystemId.ToString());
  return *this;
}

}
