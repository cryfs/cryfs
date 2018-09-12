#include "OuterEncryptor.h"
#include <cpp-utils/crypto/RandomPadding.h>
#include "OuterConfig.h"

using std::string;
using cpputils::Data;
using cpputils::RandomPadding;
using boost::optional;
using boost::none;
using namespace cpputils::logging;

namespace cryfs {
    OuterEncryptor::OuterEncryptor(Cipher::EncryptionKey key, cpputils::Data kdfParameters)
            : _key(std::move(key)), _kdfParameters(std::move(kdfParameters)) {
    }

    OuterConfig OuterEncryptor::encrypt(const Data &plaintext) const {
        auto padded = RandomPadding::add(plaintext, CONFIG_SIZE);
        auto ciphertext = Cipher::encrypt(static_cast<const uint8_t*>(padded.data()), padded.size(), _key);
        return OuterConfig{_kdfParameters.copy(), std::move(ciphertext), false};
    }

    optional<Data> OuterEncryptor::decrypt(const OuterConfig &outerConfig) const {
        ASSERT(outerConfig.kdfParameters == _kdfParameters, "OuterEncryptor was initialized with wrong key config");
        auto inner = Cipher::decrypt(static_cast<const uint8_t*>(outerConfig.encryptedInnerConfig.data()), outerConfig.encryptedInnerConfig.size(), _key);
        if(inner == none) {
            return none;
        }
        return RandomPadding::remove(*inner);
    }
}
