#include "CryConfigEncryptor.h"
#include <messmer/cpp-utils/crypto/RandomPadding.h>
#include "OuterConfig.h"

using std::string;
using cpputils::unique_ref;
using cpputils::Data;
using cpputils::RandomPadding;
using cpputils::DerivedKeyConfig;
using boost::optional;
using boost::none;
using namespace cpputils::logging;

namespace cryfs {
    CryConfigEncryptor::CryConfigEncryptor(unique_ref<InnerEncryptor> innerEncryptor, OuterCipher::EncryptionKey outerKey, DerivedKeyConfig keyConfig)
            : _innerEncryptor(std::move(innerEncryptor)), _outerKey(std::move(outerKey)), _keyConfig(std::move(keyConfig)) {
    }

    Data CryConfigEncryptor::encrypt(const Data &plaintext) {
        auto inner = _innerEncryptor->encrypt(plaintext);
        auto padded = RandomPadding::add(inner, CONFIG_SIZE);
        auto ciphertext = OuterCipher::encrypt(static_cast<const uint8_t*>(padded.data()), padded.size(), _outerKey);
        return OuterConfig{_keyConfig, std::move(ciphertext)}.serialize();
    }

    optional<Data> CryConfigEncryptor::decrypt(const Data &data) {
        auto outerConfig = OuterConfig::deserialize(data);
        if (outerConfig == none) {
            return none;
        }
        return _decryptInnerConfig(outerConfig->encryptedInnerConfig);
    }

    optional<Data> CryConfigEncryptor::_decryptInnerConfig(const Data &encryptedInnerConfig) {
        auto inner = OuterCipher::decrypt(static_cast<const uint8_t*>(encryptedInnerConfig.data()), encryptedInnerConfig.size(), _outerKey);
        if(inner == none) {
            return none;
        }
        auto depadded = RandomPadding::remove(*inner);
        if(depadded == none) {
            return none;
        }
        return _innerEncryptor->decrypt(*depadded);
    }
}
