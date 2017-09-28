#include "BasedirMetadata.h"
#include <boost/property_tree/ptree.hpp>
#include <boost/property_tree/json_parser.hpp>
#include <cryptopp/sha.h>
#include <boost/filesystem/operations.hpp>
#include "LocalStateDir.h"

namespace bf = boost::filesystem;
using boost::property_tree::ptree;
using boost::property_tree::write_json;
using boost::property_tree::read_json;
using boost::optional;
using boost::none;
using std::ostream;
using std::istream;
using std::ifstream;
using std::ofstream;

namespace cryfs {

namespace {
bf::path _localStateConfigFile(const bf::path& basedir) {
  std::string basedir_id;
  CryptoPP::SHA512 hash;
  CryptoPP::StringSource(bf::canonical(basedir).native(), true,
      new CryptoPP::HashFilter(hash,
          new CryptoPP::HexEncoder(
              new CryptoPP::StringSink(basedir_id)
          )
      )
  );
  return LocalStateDir::forMapFromBasedirToConfigFiles() / basedir_id;
}

void _serialize(ostream& stream, const CryConfig::FilesystemID& filesystemId) {
  ptree pt;
  pt.put<std::string>("filesystemId", filesystemId.ToString());

  write_json(stream, pt);
}

CryConfig::FilesystemID _deserialize(istream& stream) {
  ptree pt;
  read_json(stream, pt);

  std::string filesystemId = pt.get<std::string>("filesystemId");

  return CryConfig::FilesystemID::FromString(filesystemId);
}

optional<CryConfig::FilesystemID> _load(const bf::path &metadataFilePath) {
  ifstream file(metadataFilePath.native());
  if (!file.good()) {
    // State file doesn't exist
    return none;
  }
  return _deserialize(file);
}

void _save(const bf::path &metadataFilePath, const CryConfig::FilesystemID& filesystemId) {
  ofstream file(metadataFilePath.native(), std::ios::trunc);
  _serialize(file, filesystemId);
}

}

bool BasedirMetadata::filesystemIdForBasedirIsCorrect(const bf::path &basedir, const CryConfig::FilesystemID &filesystemId) {
  auto metadataFile = _localStateConfigFile(basedir);
  auto loaded = _load(metadataFile);
  if (loaded == none) {
    // Local state not known. Possibly the file system is currently being created.
    return true;
  }
  return loaded == filesystemId;
}

void BasedirMetadata::updateFilesystemIdForBasedir(const bf::path &basedir, const CryConfig::FilesystemID &filesystemId) {
  auto metadataFile = _localStateConfigFile(basedir);
  _save(metadataFile, filesystemId);
}

}
