#include "CryConfigFile.h"
#include <fstream>
#include <boost/filesystem.hpp>
#include <sstream>
#include <cpp-utils/logging/logging.h>

using boost::optional;
using boost::none;
using std::ifstream;
using std::ofstream;
using std::string;
using std::istringstream;
using std::ostringstream;
using std::stringstream;
using std::istream;
using cpputils::Data;
using cpputils::unique_ref;
using cpputils::SCryptSettings;
namespace bf = boost::filesystem;
using namespace cpputils::logging;

namespace cryfs {

CryConfigFile::~CryConfigFile() {
    //We do not call save() here, because we do not want the config file to be re-encrypted on each filesystem run
}

optional<CryConfigFile> CryConfigFile::load(const bf::path &path, const string &password) {
    auto encryptedConfigData = Data::LoadFromFile(path);
    if (encryptedConfigData == none) {
        LOG(ERROR) << "Config file not found";
        return none;
    }
    auto encryptor = CryConfigEncryptorFactory::loadKey(*encryptedConfigData, password);
    if (encryptor == none) {
        return none;
    }
    auto decrypted = (*encryptor)->decrypt(*encryptedConfigData);
    if (decrypted == none) {
        return none;
    }
    CryConfig config = CryConfig::load(decrypted->data);
    if (config.Cipher() != decrypted->cipherName) {
        LOG(ERROR) << "Inner cipher algorithm used to encrypt config file doesn't match config value";
        return none;
    }
    auto configFile = CryConfigFile(path, std::move(config), std::move(*encryptor));
    if (decrypted->wasInDeprecatedConfigFormat) {
        // Migrate it to new format
        configFile.save();
    }
    //TODO For newer compilers, this works without std::move
    return std::move(configFile);
}

CryConfigFile CryConfigFile::create(const bf::path &path, CryConfig config, const string &password, const SCryptSettings &scryptSettings) {
    if (bf::exists(path)) {
        throw std::runtime_error("Config file exists already.");
    }
    auto result = CryConfigFile(path, std::move(config), CryConfigEncryptorFactory::deriveKey(password, scryptSettings));
    result.save();
    return result;
}

CryConfigFile::CryConfigFile(const bf::path &path, CryConfig config, unique_ref<CryConfigEncryptor> encryptor)
    : _path (path), _config(std::move(config)), _encryptor(std::move(encryptor)) {
}

void CryConfigFile::save() const {
    Data configData = _config.save();
    auto encrypted = _encryptor->encrypt(configData, _config.Cipher());
    encrypted.StoreToFile(_path);
}

CryConfig *CryConfigFile::config() {
    return &_config;
}

}
