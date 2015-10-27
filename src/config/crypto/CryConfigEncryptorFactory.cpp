#include "CryConfigEncryptorFactory.h"
#include <messmer/blockstore/implementations/encrypted/ciphers/ciphers.h>

using namespace cpputils::logging;
using boost::optional;
using boost::none;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Data;
using cpputils::Deserializer;
using std::string;

namespace cryfs {

    constexpr size_t CryConfigEncryptorFactory::OuterKeySize;

    optional<unique_ref<CryConfigEncryptor>> CryConfigEncryptorFactory::loadKey(const Data &ciphertext,
                                                                                const string &password) {
        using Cipher = blockstore::encrypted::AES256_GCM; //TODO Allow other ciphers
        Deserializer deserializer(&ciphertext);
        try {
            CryConfigEncryptor::checkHeader(&deserializer);
            auto derivedKey = _loadKey<Cipher>(&deserializer, password);
            auto outerKey = derivedKey.key().take<OuterKeySize>();
            auto innerKey = derivedKey.key().drop<OuterKeySize>();
            return make_unique_ref<CryConfigEncryptor>(
                       make_unique_ref<ConcreteInnerEncryptor<Cipher>>(innerKey),
                       outerKey,
                       derivedKey.moveOutConfig()
                   );
        } catch (const std::exception &e) {
            LOG(ERROR) << "Error loading configuration: " << e.what();
            return none; // This can be caused by invalid loaded data and is not necessarily a programming logic error. Don't throw exception.
        }
    }
}
