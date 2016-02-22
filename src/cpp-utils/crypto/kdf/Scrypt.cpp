#include "Scrypt.h"

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

    void SCrypt::derive(void *destination, size_t size, const string &password) {
        _checkCallOnlyOnce();
        int errorcode = crypto_scrypt(reinterpret_cast<const uint8_t*>(password.c_str()), password.size(),
                                      reinterpret_cast<const uint8_t*>(_config.salt().data()), _config.salt().size(),
                                      _config.N(), _config.r(), _config.p(),
                                      static_cast<uint8_t*>(destination), size);
        if (errorcode != 0) {
            throw std::runtime_error("Error running scrypt key derivation.");
        }
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