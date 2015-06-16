#include "CryConfigLoader.h"
#include <boost/filesystem.hpp>

namespace bf = boost::filesystem;
using std::unique_ptr;
using std::make_unique;

namespace cryfs {

unique_ptr<CryConfig> CryConfigLoader::loadOrCreate(const bf::path &filename) {
  auto config = loadExisting(filename);
  if (config.get() != nullptr) {
    return config;
  }
  return createNew(filename);
}

unique_ptr<CryConfig> CryConfigLoader::createNew(const bf::path &filename) {
  auto config = make_unique<CryConfig>(filename);
  _initializeConfig(config.get());
  config->save();
  return config;
}

void CryConfigLoader::_initializeConfig(CryConfig *config) {
  _generateEncKey(config);
  _generateRootBlobKey(config);
}

void CryConfigLoader::_generateEncKey(CryConfig *config) {
  printf("Generating secure encryption key...");
  fflush(stdout);
  auto new_key = Cipher::EncryptionKey::CreateOSRandom();
  config->SetEncryptionKey(new_key.ToString());
  printf("done\n");
  fflush(stdout);
}

void CryConfigLoader::_generateRootBlobKey(CryConfig *config) {
  //An empty root blob entry will tell CryDevice to create a new root blob
  config->SetRootBlob("");
}

unique_ptr<CryConfig> CryConfigLoader::loadExisting(const bf::path &filename) {
  if (bf::exists(filename)) {
    return make_unique<CryConfig>(filename);
  }
  return nullptr;
}

}
