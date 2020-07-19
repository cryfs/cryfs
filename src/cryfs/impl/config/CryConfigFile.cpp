#include "CryConfigFile.h"
#include <fstream>
#include <boost/filesystem.hpp>
#include <sstream>
#include <cpp-utils/logging/logging.h>

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
using cpputils::make_unique_ref;
using cpputils::either;
namespace bf = boost::filesystem;
using namespace cpputils::logging;

namespace cryfs {

CryConfigFile::~CryConfigFile() {
    //We do not call save() here, because we do not want the config file to be re-encrypted on each filesystem run
}

either<CryConfigFile::LoadError, unique_ref<CryConfigFile>> CryConfigFile::load(bf::path path, CryKeyProvider* keyProvider, Access access) {
    auto encryptedConfigData = Data::LoadFromFile(path);
    if (encryptedConfigData == none) {
        return LoadError::ConfigFileNotFound;
    }
    auto encryptor = CryConfigEncryptorFactory::loadExistingKey(*encryptedConfigData, keyProvider);
    if (encryptor == none) {
        return LoadError::DecryptionFailed;
    }
    auto decrypted = (*encryptor)->decrypt(*encryptedConfigData);
    if (decrypted == none) {
        return LoadError::DecryptionFailed;
    }
    CryConfig config = CryConfig::load(decrypted->data);
    if (config.Cipher() != decrypted->cipherName) {
        LOG(ERR, "Inner cipher algorithm used to encrypt config file doesn't match config value");
        return LoadError::DecryptionFailed;
    }
    auto configFile = make_unique_ref<CryConfigFile>(CryConfigFile(std::move(path), std::move(config), std::move(*encryptor), access));
    if (decrypted->wasInDeprecatedConfigFormat) {
        if (access == Access::ReadWrite) {
            // Migrate it to new format
            configFile->save();
        }
    }
    #if !defined(__clang__) && !defined(_MSC_VER) && defined(__GNUC__) && __GNUC__ < 8
        return std::move(configFile);
    #else
        return configFile;
    #endif
}

unique_ref<CryConfigFile> CryConfigFile::create(bf::path path, CryConfig config, CryKeyProvider* keyProvider) {
    if (bf::exists(path)) {
        throw std::runtime_error("Config file exists already.");
    }
    auto result = make_unique_ref<CryConfigFile>(std::move(path), std::move(config), CryConfigEncryptorFactory::deriveNewKey(keyProvider), CryConfigFile::Access::ReadWrite);
    result->save();
    return result;
}

CryConfigFile::CryConfigFile(bf::path path, CryConfig config, unique_ref<CryConfigEncryptor> encryptor, Access access)
    : _path(std::move(path)), _config(std::move(config)), _encryptor(std::move(encryptor)), _access(access) {
}

void CryConfigFile::save() const {
    if (_access == Access::ReadOnly) {
        throw std::logic_error("Tried to save the cryfs.config file while being in read only mode");
    }
    Data configData = _config.save();
    auto encrypted = _encryptor->encrypt(configData, _config.Cipher());
    encrypted.StoreToFile(_path);
}

CryConfig *CryConfigFile::config() {
    return const_cast<CryConfig*>(const_cast<const CryConfigFile*>(this)->config());
}

const CryConfig *CryConfigFile::config() const {
    return &_config;
}

}
