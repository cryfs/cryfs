#include "CryConfigLoader.h"
#include <boost/filesystem.hpp>

namespace bf = boost::filesystem;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Console;
using cpputils::IOStreamConsole;
using boost::optional;
using boost::none;
using std::vector;
using std::string;

namespace cryfs {

CryConfigLoader::CryConfigLoader(): CryConfigLoader(make_unique_ref<IOStreamConsole>()) {}

CryConfigLoader::CryConfigLoader(unique_ref<Console> console) : _console(std::move(console)) {}

unique_ref<CryConfig> CryConfigLoader::loadOrCreate(const bf::path &filename) {
  auto config = loadExisting(filename);
  if (config != none) {
    return std::move(*config);
  }
  return createNew(filename);
}

unique_ref<CryConfig> CryConfigLoader::createNew(const bf::path &filename) {
  auto config = make_unique_ref<CryConfig>(filename);
  _initializeConfig(config.get());
  config->save();
  return config;
}

void CryConfigLoader::_initializeConfig(CryConfig *config) {
  _generateCipher(config);
  _generateEncKey(config);
  _generateRootBlobKey(config);
}

void CryConfigLoader::_initializeConfigWithWeakKey(CryConfig *config) {
  _generateTestCipher(config);
  _generateWeakEncKey(config);
  _generateRootBlobKey(config);
}

void CryConfigLoader::_generateCipher(CryConfig *config) {
  vector<string> ciphers = CryCiphers::supportedCipherNames();
  string cipherName = "";
  bool askAgain = true;
  while(askAgain) {
    int cipherIndex = _console->ask("Which block cipher do you want to use?", ciphers);
    cipherName = ciphers[cipherIndex];
    askAgain = !_showWarningForCipherAndReturnIfOk(cipherName);
  };
  config->SetCipher(cipherName);
}

bool CryConfigLoader::_showWarningForCipherAndReturnIfOk(const string &cipherName) {
  auto warning = CryCiphers::find(cipherName).warning();
  if (warning == boost::none) {
    return true;
  }
  return _console->askYesNo(string() + (*warning) + " Do you want to take this cipher nevertheless?");
}

void CryConfigLoader::_generateEncKey(CryConfig *config) {
  _console->print("\nGenerating secure encryption key...");
  config->SetEncryptionKey(CryCiphers::find(config->Cipher()).createKey());
  _console->print("done\n");
}

void CryConfigLoader::_generateTestCipher(CryConfig *config) {
  config->SetCipher("aes-256-gcm");
}

void CryConfigLoader::_generateWeakEncKey(CryConfig *config) {
  auto new_key = blockstore::encrypted::AES256_GCM::EncryptionKey::CreatePseudoRandom();
  config->SetEncryptionKey(new_key.ToString());
}

void CryConfigLoader::_generateRootBlobKey(CryConfig *config) {
  //An empty root blob entry will tell CryDevice to create a new root blob
  config->SetRootBlob("");
}

optional<unique_ref<CryConfig>> CryConfigLoader::loadExisting(const bf::path &filename) {
  if (bf::exists(filename)) {
    return make_unique_ref<CryConfig>(filename);
  }
  return none;
}

unique_ref<CryConfig> CryConfigLoader::loadOrCreateWithWeakKey(const bf::path &filename) {
  auto config = loadExisting(filename);
  if (config != none) {
    return std::move(*config);
  }
  return createNewWithWeakKey(filename);
}

unique_ref<CryConfig> CryConfigLoader::createNewWithWeakKey(const bf::path &filename) {
  auto config = make_unique_ref<CryConfig>(filename);
  _initializeConfigWithWeakKey(config.get());
  config->save();
  return config;
}

}
