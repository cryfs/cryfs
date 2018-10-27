#include "CryConfigEncryptorFactory.h"
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include "outer/OuterConfig.h"
#include "cryfs/config/CryKeyProvider.h"

using namespace cpputils::logging;
using boost::optional;
using boost::none;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Data;
using std::string;

//TODO It would be better, not to generate a MaxTotalKeySize key here, but to generate the outer key first, and then
//     (once we know which inner cipher was used) only generate as many key bytes as we need for the inner cipher.
//     This would need a change in the scrypt interface though, because right now we can't continue past key computations.
//TODO I might be able to know the actual key size here (at runtime) and switch the SCrypt deriveKey() interface to getting a dynamic size.

namespace cryfs {

    optional<unique_ref<CryConfigEncryptor>> CryConfigEncryptorFactory::loadExistingKey(const Data &data,
                                                                                CryKeyProvider *keyProvider) {
        auto outerConfig = OuterConfig::deserialize(data);
        if (outerConfig == none) {
            return none;
        }
        auto key = keyProvider->requestKeyForExistingFilesystem(CryConfigEncryptor::MaxTotalKeySize, outerConfig->kdfParameters);
        return make_unique_ref<CryConfigEncryptor>(key, std::move(outerConfig->kdfParameters));
    }

    unique_ref<CryConfigEncryptor> CryConfigEncryptorFactory::deriveNewKey(CryKeyProvider *keyProvider) {
        auto keyResult = keyProvider->requestKeyForNewFilesystem(CryConfigEncryptor::MaxTotalKeySize);
        return make_unique_ref<CryConfigEncryptor>(std::move(keyResult.key), std::move(keyResult.kdfParameters));
    }
}
