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
namespace bf = boost::filesystem;
using namespace cpputils::logging;

namespace cryfs {

CryConfigFile CryConfigFile::create(const bf::path &path, CryConfig config, const string &password) {
    if (bf::exists(path)) {
        throw std::runtime_error("Config file exists already.");
    }
    auto configEncKey = Encryptor::deriveKey(password);
    auto result = CryConfigFile(path, std::move(config), std::move(configEncKey));
    result.save();
    return result;
}

CryConfigFile::~CryConfigFile() {
    //We do not call save() here, because we do not want the config file to be re-encrypted on each filesystem run
}

optional<CryConfigFile> CryConfigFile::load(const bf::path &path, const string &password) {
    auto encryptedConfigData = Data::LoadFromFile(path);
    if (encryptedConfigData == none) {
        LOG(ERROR) << "Config file not found";
        return none;
    }
    auto decrypted = Encryptor::decrypt(*encryptedConfigData, password);
    if (decrypted == none) {
        return none;
    }
    CryConfig config = CryConfig::load(decrypted->second);
    return CryConfigFile(path, std::move(config), std::move(decrypted->first));
}

CryConfigFile::CryConfigFile(const bf::path &path, CryConfig config, ConfigEncryptionKey configEncKey)
    : _path (path), _config(std::move(config)), _configEncKey(std::move(configEncKey)) {
}

void CryConfigFile::save() const {
    Data configData = _config.save();
    auto encrypted = Encryptor::encrypt(configData, _configEncKey);
    encrypted.StoreToFile(_path);
}

CryConfig *CryConfigFile::config() {
    return &_config;
}

}
