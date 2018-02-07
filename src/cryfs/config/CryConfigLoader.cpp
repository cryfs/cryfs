#include "CryConfigLoader.h"
#include "CryConfigFile.h"
#include <boost/filesystem.hpp>
#include <cpp-utils/random/Random.h>
#include <cpp-utils/logging/logging.h>
#include <boost/algorithm/string/predicate.hpp>
#include <gitversion/gitversion.h>
#include <gitversion/VersionCompare.h>
#include "../CryfsException.h"

namespace bf = boost::filesystem;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Console;
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

CryConfigLoader::CryConfigLoader(shared_ptr<Console> console, RandomGenerator &keyGenerator, const SCryptSettings &scryptSettings, function<string()> askPasswordForExistingFilesystem, function<string()> askPasswordForNewFilesystem, const optional<string> &cipherFromCommandLine, const boost::optional<uint32_t> &blocksizeBytesFromCommandLine)
    : _console(console), _creator(console, keyGenerator), _scryptSettings(scryptSettings),
      _askPasswordForExistingFilesystem(askPasswordForExistingFilesystem), _askPasswordForNewFilesystem(askPasswordForNewFilesystem),
      _cipherFromCommandLine(cipherFromCommandLine), _blocksizeBytesFromCommandLine(blocksizeBytesFromCommandLine) {
}

optional<CryConfigFile> CryConfigLoader::_loadConfig(const bf::path &filename, bool allowFilesystemUpgrade) {
  string password = _askPasswordForExistingFilesystem();
  std::cout << "Loading config file (this can take some time)..." << std::flush;
  auto config = CryConfigFile::load(filename, password);
  if (config == none) {
    return none;
  }
  std::cout << "done" << std::endl;
#ifndef CRYFS_NO_COMPATIBILITY
  //Since 0.9.7 and 0.9.8 set their own version to cryfs.version instead of the filesystem format version (which is 0.9.6), overwrite it
  if (config->config()->Version() == "0.9.7" || config->config()->Version() == "0.9.8") {
    config->config()->SetVersion("0.9.6");
  }
#endif
  _checkVersion(*config->config(), allowFilesystemUpgrade);
#ifndef CRYFS_NO_COMPATIBILITY
  //Since 0.9.3-alpha set the config value cryfs.blocksizeBytes wrongly to 32768 (but didn't use the value), we have to fix this here.
  if (config->config()->Version() != "0+unknown" && VersionCompare::isOlderThan(config->config()->Version(), "0.9.3-rc1")) {
    config->config()->SetBlocksizeBytes(32832);
  }
#endif
  if (config->config()->Version() != CryConfig::FilesystemFormatVersion) {
    config->config()->SetVersion(CryConfig::FilesystemFormatVersion);
    config->save();
  }
  if (config->config()->LastOpenedWithVersion() != gitversion::VersionString()) {
    config->config()->SetLastOpenedWithVersion(gitversion::VersionString());
    config->save();
  }
  _checkCipher(*config->config());
  return std::move(*config);
}

void CryConfigLoader::_checkVersion(const CryConfig &config, bool allowFilesystemUpgrade) {
  if (gitversion::VersionCompare::isOlderThan(CryConfig::FilesystemFormatVersion, config.Version())) {
    if (!_console->askYesNo("This filesystem is for CryFS " + config.Version() + " or later and should not be opened with older versions. It is strongly recommended to update your CryFS version. However, if you have backed up your base directory and know what you're doing, you can continue trying to load it. Do you want to continue?", false)) {
      throw CryfsException("This filesystem is for CryFS " + config.Version() + " or later. Please update your CryFS version.", ErrorCode::TooNewFilesystemFormat);
    }
  }
  if (!allowFilesystemUpgrade && gitversion::VersionCompare::isOlderThan(config.Version(), CryConfig::FilesystemFormatVersion)) {
    if (!_console->askYesNo("This filesystem is for CryFS " + config.Version() + " (or a later version with the same storage format). You're running a CryFS version using storage format " + CryConfig::FilesystemFormatVersion + ". It can be migrated, but afterwards couldn't be opened anymore with older versions. Do you want to migrate it?", false)) {
      throw CryfsException("This filesystem is for CryFS " + config.Version() + " (or a later version with the same storage format). It has to be migrated.", ErrorCode::TooOldFilesystemFormat);
    }
  }
}

void CryConfigLoader::_checkCipher(const CryConfig &config) const {
  if (_cipherFromCommandLine != none && config.Cipher() != *_cipherFromCommandLine) {
    throw CryfsException(string() + "Filesystem uses " + config.Cipher() + " cipher and not " + *_cipherFromCommandLine + " as specified.", ErrorCode::WrongCipher);
  }
}

optional<CryConfigFile> CryConfigLoader::loadOrCreate(const bf::path &filename, bool allowFilesystemUpgrade) {
  if (bf::exists(filename)) {
    return _loadConfig(filename, allowFilesystemUpgrade);
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
