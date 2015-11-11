#include "CryConfigEncryptorFactory.h"
#include <messmer/cpp-utils/crypto/symmetric/ciphers.h>
#include "OuterConfig.h"

using namespace cpputils::logging;
using boost::optional;
using boost::none;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Data;
using cpputils::DerivedKey;
using cpputils::DerivedKeyConfig;
using cpputils::SCrypt;
using std::string;

namespace cryfs {

    constexpr size_t CryConfigEncryptorFactory::OuterKeySize;
    constexpr size_t CryConfigEncryptorFactory::MaxTotalKeySize;

    optional<unique_ref<CryConfigEncryptor>> CryConfigEncryptorFactory::loadKey(const Data &data,
                                                                                const string &password) {
        using Cipher = cpputils::AES256_GCM; //TODO Allow other ciphers

        auto outerConfig = OuterConfig::deserialize(data);
        if (outerConfig == none) {
            return none;
        }
        auto derivedKey = _deriveKey(outerConfig->keyConfig, password);
        auto outerKey = derivedKey.key().take<OuterKeySize>();
        auto innerKey = derivedKey.key().drop<OuterKeySize>().take<Cipher::EncryptionKey::BINARY_LENGTH>();
        return make_unique_ref<CryConfigEncryptor>(
                make_unique_ref<ConcreteInnerEncryptor<Cipher>>(innerKey),
                outerKey,
                derivedKey.moveOutConfig()
        );
    }

    cpputils::DerivedKey<CryConfigEncryptorFactory::MaxTotalKeySize>
    CryConfigEncryptorFactory::_deriveKey(const DerivedKeyConfig &keyConfig, const std::string &password) {
        //TODO It would be better, not to generate a MaxTotalKeySize key here, but to generate the outer key first, and then
        //     (once we know which inner cipher was used) only generate as many key bytes as we need for the inner cipher.
        //     This would need a change in the scrypt interface though, because right now we can't continue past key computations.
        auto key = SCrypt().generateKeyFromConfig<MaxTotalKeySize>(password, keyConfig);
        return DerivedKey<MaxTotalKeySize>(keyConfig, std::move(key));
    }
}
