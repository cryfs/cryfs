#include "CryConfigFile.h"
#include <fstream>
#include <boost/filesystem.hpp>
#include <sstream>
#include "crypto/Scrypt.h"
#include <messmer/cpp-utils/logging/logging.h>

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
namespace bf = boost::filesystem;
using namespace cpputils::logging;

namespace cryfs {

CryConfigFile::~CryConfigFile() {
    //We do not call save() here, because we do not want the config file to be re-encrypted on each filesystem run
}

CryConfigFile CryConfigFile::create(const bf::path &path, CryConfig config, const string &password) {
    using ConfigCipher = blockstore::encrypted::AES256_GCM; // TODO Take cipher from config instead
    if (bf::exists(path)) {
        throw std::runtime_error("Config file exists already.");
    }
    auto result = CryConfigFile(path, std::move(config), CryConfigEncryptor::deriveKey<ConfigCipher>(password));
    result.save();
    return result;
}

optional<CryConfigFile> CryConfigFile::load(const bf::path &path, const string &password) {
    auto encryptedConfigData = Data::LoadFromFile(path);
    if (encryptedConfigData == none) {
        LOG(ERROR) << "Config file not found";
        return none;
    }
    auto encryptor = CryConfigEncryptor::loadKey(*encryptedConfigData, password);
    if (encryptor == none) {
        return none;
    }
    auto decrypted = (*encryptor)->decrypt(*encryptedConfigData);
    if (decrypted == none) {
        return none;
    }
    CryConfig config = CryConfig::load(*decrypted);
    return CryConfigFile(path, std::move(config), std::move(*encryptor));
}

CryConfigFile::CryConfigFile(const bf::path &path, CryConfig config, unique_ref<CryConfigEncryptor> encryptor)
    : _path (path), _config(std::move(config)), _encryptor(std::move(encryptor)) {
}

void CryConfigFile::save() const {
    Data configData = _config.save();
    auto encrypted = _encryptor->encrypt(configData);
    encrypted.StoreToFile(_path);
}

CryConfig *CryConfigFile::config() {
    return &_config;
}

}
