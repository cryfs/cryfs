#include "CryConfigLoader.h"
#include "CryConfigFile.h"
#include <boost/filesystem.hpp>
#include <cpp-utils/random/Random.h>
#include <cpp-utils/logging/logging.h>
#include <boost/algorithm/string/predicate.hpp>
#include <gitversion/gitversion.h>
#include <gitversion/VersionCompare.h>

namespace bf = boost::filesystem;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Console;
using cpputils::IOStreamConsole;
using cpputils::Random;
using cpputils::RandomGenerator;
using cpputils::SCryptSettings;
using boost::optional;
using boost::none;
using std::shared_ptr;
using std::vector;
using std::string;
using std::function;
using std::shared_ptr;
using gitversion::VersionCompare;
using namespace cpputils::logging;

namespace cryfs {

CryConfigLoader::CryConfigLoader(shared_ptr<Console> console, RandomGenerator &keyGenerator, const SCryptSettings &scryptSettings, function<string()> askPasswordForExistingFilesystem, function<string()> askPasswordForNewFilesystem, const optional<string> &cipherFromCommandLine, const boost::optional<uint32_t> &blocksizeBytesFromCommandLine, bool noninteractive)
    : _creator(std::move(console), keyGenerator, noninteractive), _scryptSettings(scryptSettings),
      _askPasswordForExistingFilesystem(askPasswordForExistingFilesystem), _askPasswordForNewFilesystem(askPasswordForNewFilesystem),
      _cipherFromCommandLine(cipherFromCommandLine), _blocksizeBytesFromCommandLine(blocksizeBytesFromCommandLine) {
}

optional<CryConfigFile> CryConfigLoader::_loadConfig(const bf::path &filename) {
  string password = _askPasswordForExistingFilesystem();
  std::cout << "Loading config file (this can take some time)..." << std::flush;
  auto config = CryConfigFile::load(filename, password);
  if (config.is_left()) {
    return none;
  }
  std::cout << "done" << std::endl;

  _checkVersion(*config.right().config());
#ifndef CRYFS_NO_COMPATIBILITY
  //Since 0.9.3-alpha set the config value cryfs.blocksizeBytes wrongly to 32768 (but didn't use the value), we have to fix this here.
  if (VersionCompare::isOlderThan(config.right().config()->Version(), "0.9.3-rc1")) {
    config.right().config()->SetBlocksizeBytes(32832);
  }
#endif
  if (config.right().config()->Version() != gitversion::VersionString()) {
    config.right().config()->SetVersion(gitversion::VersionString());
    config.right().save();
  }
  _checkCipher(*config.right().config());
  return std::move(config.right());
}

void CryConfigLoader::_checkVersion(const CryConfig &config) {
  const string allowedVersionPrefix = string() + gitversion::MajorVersion() + "." + gitversion::MinorVersion() + ".";
  if (!boost::starts_with(config.Version(), allowedVersionPrefix)) {
    throw std::runtime_error(string() + "This filesystem was created with CryFS " + config.Version() + " and is incompatible. Please create a new one with your version of CryFS and migrate your data.");
  }
}

void CryConfigLoader::_checkCipher(const CryConfig &config) const {
  if (_cipherFromCommandLine != none && config.Cipher() != *_cipherFromCommandLine) {
    throw std::runtime_error(string() + "Filesystem uses " + config.Cipher() + " cipher and not " + *_cipherFromCommandLine + " as specified.");
  }
}

optional<CryConfigFile> CryConfigLoader::loadOrCreate(const bf::path &filename) {
  if (bf::exists(filename)) {
    return _loadConfig(filename);
  } else {
    return _createConfig(filename);
  }
}

CryConfigFile CryConfigLoader::_createConfig(const bf::path &filename) {
  auto config = _creator.create(_cipherFromCommandLine, _blocksizeBytesFromCommandLine);
  //TODO Ask confirmation if using insecure password (<8 characters)
  string password = _askPasswordForNewFilesystem();
  std::cout << "Creating config file (this can take some time)..." << std::flush;
  auto result = CryConfigFile::create(filename, std::move(config), password, _scryptSettings);
  std::cout << "done" << std::endl;
  return result;
}


}
