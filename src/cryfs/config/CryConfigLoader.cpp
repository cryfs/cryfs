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
    : _console(console), _creator(console, keyGenerator, noninteractive), _scryptSettings(scryptSettings),
      _askPasswordForExistingFilesystem(askPasswordForExistingFilesystem), _askPasswordForNewFilesystem(askPasswordForNewFilesystem),
      _cipherFromCommandLine(cipherFromCommandLine), _blocksizeBytesFromCommandLine(blocksizeBytesFromCommandLine) {
}

optional<CryConfigFile> CryConfigLoader::_loadConfig(const bf::path &filename) {
  string password = _askPasswordForExistingFilesystem();
  std::cout << "Loading config file (this can take some time)..." << std::flush;
  auto config = CryConfigFile::load(filename, password);
  if (config == none) {
    return none;
  }
  std::cout << "done" << std::endl;
  _checkVersion(*config->config());
#ifndef CRYFS_NO_COMPATIBILITY
  //Since 0.9.3-alpha set the config value cryfs.blocksizeBytes wrongly to 32768 (but didn't use the value), we have to fix this here.
  if (config->config()->Version() != "0+unknown" && VersionCompare::isOlderThan(config->config()->Version(), "0.9.3-rc1")) {
    config->config()->SetBlocksizeBytes(32832);
  }
#endif
  if (config->config()->Version() != gitversion::VersionString()) {
    config->config()->SetVersion(gitversion::VersionString());
    config->save();
  }
  _checkCipher(*config->config());
  return std::move(*config);
}

void CryConfigLoader::_checkVersion(const CryConfig &config) {
  if (gitversion::VersionCompare::isOlderThan(gitversion::VersionString(), config.Version())) {
    if (!_console->askYesNo("This filesystem is for CryFS " + config.Version() + " and should not be opened with older versions. It is strongly recommended to update your CryFS version. However, if you have backed up your base directory and know what you're doing, you can continue trying to load it. Do you want to continue?")) {
      throw std::runtime_error("Not trying to load file system.");
    }
  }
  if (gitversion::VersionCompare::isOlderThan(config.Version(), gitversion::VersionString())) {
    if (!_console->askYesNo("This filesystem is for CryFS " + config.Version() + ". It can be migrated to CryFS " + gitversion::VersionString() + ", but afterwards couldn't be opened anymore with older versions. Do you want to migrate it?")) {
      throw std::runtime_error(string() + "Not migrating file system.");
    }
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
