#include "CryConfigLoader.h"
#include <boost/filesystem.hpp>

namespace bf = boost::filesystem;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
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
  vector<string> ciphers = {"aes-256-gcm", "aes-256-cfb"};
  int cipherIndex = _console->ask("Which block cipher do you want to use?", ciphers);
  config->SetCipher(ciphers[cipherIndex]);
}

void CryConfigLoader::_generateTestCipher(CryConfig *config) {
  config->SetCipher("aes-256-gcm");
}

void CryConfigLoader::_generateEncKey(CryConfig *config) {
  _console->print("Generating secure encryption key...");
  auto new_key = Cipher::EncryptionKey::CreateOSRandom();
  config->SetEncryptionKey(new_key.ToString());
  _console->print("done");
}

void CryConfigLoader::_generateWeakEncKey(CryConfig *config) {
  auto new_key = Cipher::EncryptionKey::CreatePseudoRandom();
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
