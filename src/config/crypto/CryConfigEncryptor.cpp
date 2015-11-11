#include "CryConfigEncryptor.h"
#include <messmer/cpp-utils/crypto/RandomPadding.h>

using std::string;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Data;
using cpputils::RandomPadding;
using cpputils::DerivedKeyConfig;
using cpputils::DerivedKey;
using cpputils::FixedSizeData;
using boost::optional;
using boost::none;
using namespace cpputils::logging;

namespace cryfs {
    constexpr size_t CryConfigEncryptor::OuterKeySize;
    constexpr size_t CryConfigEncryptor::MaxTotalKeySize;

    CryConfigEncryptor::CryConfigEncryptor(DerivedKey<MaxTotalKeySize> derivedKey)
            : _derivedKey(std::move(derivedKey)) {
    }

    Data CryConfigEncryptor::encrypt(const Data &plaintext, const string &cipherName) const {
        InnerConfig innerConfig = _innerEncryptor(cipherName)->encrypt(plaintext);
        Data serializedInnerConfig = innerConfig.serialize();
        OuterConfig outerConfig = _outerEncryptor()->encrypt(serializedInnerConfig);
        return outerConfig.serialize();
    }

    optional<CryConfigEncryptor::Decrypted> CryConfigEncryptor::decrypt(const Data &data) const {
        auto innerConfig = _loadInnerConfig(data);
        if (innerConfig == none) {
            return none;
        }
        auto plaintext = _innerEncryptor(innerConfig->cipherName)->decrypt(*innerConfig);
        if (plaintext == none) {
            return none;
        }
        return Decrypted{std::move(*plaintext), innerConfig->cipherName};
    }

    optional<InnerConfig> CryConfigEncryptor::_loadInnerConfig(const Data &data) const {
        auto outerConfig = OuterConfig::deserialize(data);
        if (outerConfig == none) {
            return none;
        }
        auto serializedInnerConfig = _outerEncryptor()->decrypt(*outerConfig);
        if(serializedInnerConfig == none) {
            return none;
        }
        return InnerConfig::deserialize(*serializedInnerConfig);
    }

    unique_ref<OuterEncryptor> CryConfigEncryptor::_outerEncryptor() const {
        auto outerKey = _derivedKey.key().take<OuterKeySize>();
        return make_unique_ref<OuterEncryptor>(outerKey, _derivedKey.config());
    }

    unique_ref<InnerEncryptor> CryConfigEncryptor::_innerEncryptor(const string &cipherName) const {
        auto innerKey = _derivedKey.key().drop<OuterKeySize>();
        return CryCiphers::find(cipherName).createInnerConfigEncryptor(innerKey);
    }
}
