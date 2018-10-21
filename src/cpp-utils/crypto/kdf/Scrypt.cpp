#include "Scrypt.h"
#include <vendor_cryptopp/scrypt.h>

using std::string;

namespace cpputils {

    constexpr SCryptSettings SCrypt::ParanoidSettings;
    constexpr SCryptSettings SCrypt::DefaultSettings;
    constexpr SCryptSettings SCrypt::TestSettings;

    unique_ref<SCrypt> SCrypt::forNewKey(const SCryptSettings &settings) {
        SCryptParameters kdfParameters(Random::PseudoRandom().get(settings.SALT_LEN), settings.N, settings.r, settings.p);
        return make_unique_ref<SCrypt>(std::move(kdfParameters));
    }

    unique_ref<SCrypt> SCrypt::forExistingKey(const Data &parameters) {
        return make_unique_ref<SCrypt>(SCryptParameters::deserialize(parameters));
    }

    SCrypt::SCrypt(SCryptParameters config)
            :_config(std::move(config)), _serializedConfig(_config.serialize()), _wasGeneratedBefore(false) {
    }

    EncryptionKey SCrypt::deriveKey(size_t keySize, const std::string &password) {
        _checkCallOnlyOnce();

        auto result = EncryptionKey::Null(keySize);

        size_t status = CryptoPP::Scrypt().DeriveKey(
            static_cast<uint8_t*>(result.data()), result.binaryLength(),
            reinterpret_cast<const uint8_t*>(password.c_str()), password.size(),
            static_cast<const uint8_t*>(_config.salt().data()), _config.salt().size(),
            _config.N(), _config.r(), _config.p()
        );
        if (status != 1) {
            throw std::runtime_error("Error running scrypt key derivation. Error code: "+std::to_string(status));
        }

        return result;
    }

    const Data &SCrypt::kdfParameters() const {
        return _serializedConfig;
    }

    void SCrypt::_checkCallOnlyOnce() {
        if (_wasGeneratedBefore) {
            throw std::runtime_error("An SCrypt instance can only generate exactly one key. Generating multiple keys would be insecure because we would use the same salt.");
        }
        _wasGeneratedBefore = true;
    }
}