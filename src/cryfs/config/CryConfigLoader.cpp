#include "CryConfigLoader.h"
#include "CryConfigFile.h"
#include <boost/filesystem.hpp>
#include <cpp-utils/random/Random.h>
#include <cpp-utils/logging/logging.h>
#include <boost/algorithm/string/predicate.hpp>
#include <gitversion/gitversion.h>
#include <gitversion/VersionCompare.h>
#include "../localstate/LocalStateDir.h"
#include "../localstate/LocalStateMetadata.h"

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
using std::unique_ptr;
using std::make_unique;
using gitversion::VersionCompare;
using namespace cpputils::logging;

namespace cryfs {

CryConfigLoader::CryConfigLoader(shared_ptr<Console> console, RandomGenerator &keyGenerator, const SCryptSettings &scryptSettings, function<string()> askPasswordForExistingFilesystem, function<string()> askPasswordForNewFilesystem, const optional<string> &cipherFromCommandLine, const boost::optional<uint32_t> &blocksizeBytesFromCommandLine, const boost::optional<bool> &missingBlockIsIntegrityViolationFromCommandLine)
    : _console(console), _creator(console, keyGenerator), _scryptSettings(scryptSettings),
      _askPasswordForExistingFilesystem(askPasswordForExistingFilesystem), _askPasswordForNewFilesystem(askPasswordForNewFilesystem),
      _cipherFromCommandLine(cipherFromCommandLine), _blocksizeBytesFromCommandLine(blocksizeBytesFromCommandLine),
      _missingBlockIsIntegrityViolationFromCommandLine(missingBlockIsIntegrityViolationFromCommandLine) {
}

optional<CryConfigLoader::ConfigLoadResult> CryConfigLoader::_loadConfig(const bf::path &filename) {
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
  auto localState = LocalStateMetadata::loadOrGenerate(LocalStateDir::forFilesystemId(config->config()->FilesystemId()), cpputils::Data::FromString(config->config()->EncryptionKey()));
  uint32_t myClientId = localState.myClientId();
  _checkMissingBlocksAreIntegrityViolations(&*config, myClientId);
  return ConfigLoadResult {std::move(*config), myClientId};
}

void CryConfigLoader::_checkVersion(const CryConfig &config) {
  if (gitversion::VersionCompare::isOlderThan(gitversion::VersionString(), config.Version())) {
    if (!_console->askYesNo("This filesystem is for CryFS " + config.Version() + " and should not be opened with older versions. It is strongly recommended to update your CryFS version. However, if you have backed up your base directory and know what you're doing, you can continue trying to load it. Do you want to continue?", false)) {
      throw std::runtime_error("This filesystem is for CryFS " + config.Version() + ". Please update your CryFS version.");
    }
  }
  if (gitversion::VersionCompare::isOlderThan(config.Version(), gitversion::VersionString())) {
    if (!_console->askYesNo("This filesystem is for CryFS " + config.Version() + ". It can be migrated to CryFS " + gitversion::VersionString() + ", but afterwards couldn't be opened anymore with older versions. Do you want to migrate it?", false)) {
      throw std::runtime_error("This filesystem is for CryFS " + config.Version() + ". It has to be migrated.");
    }
  }
}

void CryConfigLoader::_checkCipher(const CryConfig &config) const {
  if (_cipherFromCommandLine != none && config.Cipher() != *_cipherFromCommandLine) {
    throw std::runtime_error(string() + "Filesystem uses " + config.Cipher() + " cipher and not " + *_cipherFromCommandLine + " as specified.");
  }
}

void CryConfigLoader::_checkMissingBlocksAreIntegrityViolations(CryConfigFile *configFile, uint32_t myClientId) {
  if (_missingBlockIsIntegrityViolationFromCommandLine == optional<bool>(true) && configFile->config()->ExclusiveClientId() == none) {
    throw std::runtime_error("You specified on the command line to treat missing blocks as integrity violations, but the file system is not setup to do that.");
  }
  if (_missingBlockIsIntegrityViolationFromCommandLine == optional<bool>(false) && configFile->config()->ExclusiveClientId() != none) {
    throw std::runtime_error("You specified on the command line to not treat missing blocks as integrity violations, but the file system is setup to do that.");
  }

  // If the file system is set up to treat missing blocks as integrity violations, but we're accessing from a different client, ask whether they want to disable the feature.
  auto exclusiveClientId = configFile->config()->ExclusiveClientId();
  if (exclusiveClientId != none && *exclusiveClientId != myClientId) {
    if (!_console->askYesNo("\nThis filesystem is setup to treat missing blocks as integrity violations and therefore only works in single-client mode. You are trying to access it from a different client.\nDo you want to disable this integrity feature and stop treating missing blocks as integrity violations?\nChoosing yes will not affect the confidentiality of your data, but in future you might not notice if an attacker deletes one of your files.", false)) {
      throw std::runtime_error("File system is in single-client mode and can only be used from the client that created it.");
    }
    configFile->config()->SetExclusiveClientId(none);
    configFile->save();
  }
}

optional<CryConfigLoader::ConfigLoadResult> CryConfigLoader::loadOrCreate(const bf::path &filename) {
  if (bf::exists(filename)) {
    return _loadConfig(filename);
  } else {
    return _createConfig(filename);
  }
}

CryConfigLoader::ConfigLoadResult CryConfigLoader::_createConfig(const bf::path &filename) {
  auto config = _creator.create(_cipherFromCommandLine, _blocksizeBytesFromCommandLine, _missingBlockIsIntegrityViolationFromCommandLine);
  //TODO Ask confirmation if using insecure password (<8 characters)
  string password = _askPasswordForNewFilesystem();
  std::cout << "Creating config file (this can take some time)..." << std::flush;
  auto result = CryConfigFile::create(filename, std::move(config.config), password, _scryptSettings);
  std::cout << "done" << std::endl;
  return ConfigLoadResult {std::move(result), config.myClientId};
}


}
