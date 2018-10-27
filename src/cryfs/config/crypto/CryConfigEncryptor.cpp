#include "CryConfigEncryptor.h"
#include <cpp-utils/crypto/RandomPadding.h>

using std::string;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Data;
using boost::optional;
using boost::none;
using namespace cpputils::logging;

namespace cryfs {
    constexpr size_t CryConfigEncryptor::OuterKeySize;
    constexpr size_t CryConfigEncryptor::MaxTotalKeySize;

    CryConfigEncryptor::CryConfigEncryptor(cpputils::EncryptionKey derivedKey, cpputils::Data kdfParameters)
            : _derivedKey(std::move(derivedKey)), _kdfParameters(std::move(kdfParameters)) {
        ASSERT(_derivedKey.binaryLength() == MaxTotalKeySize, "Wrong key size");
    }

    Data CryConfigEncryptor::encrypt(const Data &plaintext, const string &cipherName) const {
        InnerConfig innerConfig = _innerEncryptor(cipherName)->encrypt(plaintext);
        Data serializedInnerConfig = innerConfig.serialize();
        OuterConfig outerConfig = _outerEncryptor()->encrypt(serializedInnerConfig);
        return outerConfig.serialize();
    }

    optional<CryConfigEncryptor::Decrypted> CryConfigEncryptor::decrypt(const Data &data) const {
        auto outerConfig = OuterConfig::deserialize(data);
        if (outerConfig == none) {
            return none;
        }
        auto serializedInnerConfig = _outerEncryptor()->decrypt(*outerConfig);
        if(serializedInnerConfig == none) {
            return none;
        }
        auto innerConfig = InnerConfig::deserialize(*serializedInnerConfig);
        if (innerConfig == none) {
            return none;
        }
        auto plaintext = _innerEncryptor(innerConfig->cipherName)->decrypt(*innerConfig);
        if (plaintext == none) {
            return none;
        }
        return Decrypted{std::move(*plaintext), innerConfig->cipherName, outerConfig->wasInDeprecatedConfigFormat};
    }

    unique_ref<OuterEncryptor> CryConfigEncryptor::_outerEncryptor() const {
        auto outerKey = _derivedKey.take(OuterKeySize);
        return make_unique_ref<OuterEncryptor>(std::move(outerKey), _kdfParameters.copy());
    }

    unique_ref<InnerEncryptor> CryConfigEncryptor::_innerEncryptor(const string &cipherName) const {
        auto innerKey = _derivedKey.drop(OuterKeySize);
        return CryCiphers::find(cipherName).createInnerConfigEncryptor(std::move(innerKey));
    }
}
