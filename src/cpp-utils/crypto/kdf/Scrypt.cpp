#include "Scrypt.h"
#include <vendor_cryptopp/scrypt.h>

using std::string;

namespace cpputils {

constexpr SCryptSettings SCrypt::ParanoidSettings;
constexpr SCryptSettings SCrypt::DefaultSettings;
constexpr SCryptSettings SCrypt::TestSettings;

namespace {
EncryptionKey _derive(size_t keySize, const std::string& password, const SCryptParameters& kdfParameters) {
    auto result = EncryptionKey::Null(keySize);

    size_t status = CryptoPP::Scrypt().DeriveKey(
        static_cast<uint8_t*>(result.data()), result.binaryLength(),
        reinterpret_cast<const uint8_t*>(password.c_str()), password.size(),
        static_cast<const uint8_t*>(kdfParameters.salt().data()), kdfParameters.salt().size(),
        kdfParameters.N(), kdfParameters.r(), kdfParameters.p()
    );
    if (status != 1) {
        throw std::runtime_error("Error running scrypt key derivation. Error code: "+std::to_string(status));
    }

    return result;
}

SCryptParameters _createNewSCryptParameters(const SCryptSettings& settings) {
    return SCryptParameters(Random::PseudoRandom().get(settings.SALT_LEN), settings.N, settings.r, settings.p);
}
}

SCrypt::SCrypt(const SCryptSettings& settingsForNewKeys)
        :_settingsForNewKeys(settingsForNewKeys) {
}

EncryptionKey SCrypt::deriveExistingKey(size_t keySize, const std::string& password, const Data& kdfParameters) {
    SCryptParameters parameters = SCryptParameters::deserialize(kdfParameters);
    auto key = _derive(keySize, password, parameters);
    return key;
}

SCrypt::KeyResult SCrypt::deriveNewKey(size_t keySize, const std::string& password) {
    SCryptParameters kdfParameters = _createNewSCryptParameters(_settingsForNewKeys);
    auto key = _derive(keySize, password, kdfParameters);
    return SCrypt::KeyResult {
        key,
        kdfParameters.serialize()
    };
}
}
