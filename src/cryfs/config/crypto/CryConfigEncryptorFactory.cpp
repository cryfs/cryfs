#include "CryConfigEncryptorFactory.h"
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include "outer/OuterConfig.h"

using namespace cpputils::logging;
using boost::optional;
using boost::none;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Data;
using cpputils::DerivedKey;
using cpputils::DerivedKeyConfig;
using cpputils::SCrypt;
using cpputils::SCryptSettings;
using std::string;

namespace cryfs {

    optional<unique_ref<CryConfigEncryptor>> CryConfigEncryptorFactory::loadKey(const Data &data,
                                                                                const string &password) {
        auto outerConfig = OuterConfig::deserialize(data);
        if (outerConfig == none) {
            return none;
        }
        auto derivedKey = _deriveKey(outerConfig->keyConfig, password);
        return make_unique_ref<CryConfigEncryptor>(std::move(derivedKey));
    }

    DerivedKey<CryConfigEncryptor::MaxTotalKeySize>
    CryConfigEncryptorFactory::_deriveKey(const DerivedKeyConfig &keyConfig, const std::string &password) {
        //TODO It would be better, not to generate a MaxTotalKeySize key here, but to generate the outer key first, and then
        //     (once we know which inner cipher was used) only generate as many key bytes as we need for the inner cipher.
        //     This would need a change in the scrypt interface though, because right now we can't continue past key computations.
        auto key = SCrypt().generateKeyFromConfig<CryConfigEncryptor::MaxTotalKeySize>(password, keyConfig);
        return DerivedKey<CryConfigEncryptor::MaxTotalKeySize>(keyConfig, std::move(key));
    }

    unique_ref<CryConfigEncryptor> CryConfigEncryptorFactory::deriveKey(const string &password, const SCryptSettings &scryptSettings) {
        auto derivedKey = cpputils::SCrypt().generateKey<CryConfigEncryptor::MaxTotalKeySize>(password, scryptSettings);
        return make_unique_ref<CryConfigEncryptor>(std::move(derivedKey));
    }
}
