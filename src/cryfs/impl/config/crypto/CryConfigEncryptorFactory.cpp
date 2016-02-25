#include "CryConfigEncryptorFactory.h"
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include "outer/OuterConfig.h"

using namespace cpputils::logging;
using boost::optional;
using boost::none;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Data;
using cpputils::FixedSizeData;
using cpputils::SCryptParameters;
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
        return _deriveKey(SCrypt::forExistingKey(outerConfig->kdfParameters), password);
    }

    unique_ref<CryConfigEncryptor> CryConfigEncryptorFactory::deriveKey(const string &password, const SCryptSettings &scryptSettings) {
        return _deriveKey(SCrypt::forNewKey(scryptSettings), password);
    }

    unique_ref<CryConfigEncryptor>
    CryConfigEncryptorFactory::_deriveKey(cpputils::unique_ref<SCrypt> kdf, const string &password) {
        //TODO It would be better, not to generate a MaxTotalKeySize key here, but to generate the outer key first, and then
        //     (once we know which inner cipher was used) only generate as many key bytes as we need for the inner cipher.
        //     This would need a change in the scrypt interface though, because right now we can't continue past key computations.
        //TODO I might be able to know the actual key size here (at runtime) and switch the SCrypt deriveKey() interface to getting a dynamic size.
        auto key = kdf->deriveKey<CryConfigEncryptor::MaxTotalKeySize>(password);
        return make_unique_ref<CryConfigEncryptor>(std::move(key), kdf->kdfParameters().copy());
    }
}
