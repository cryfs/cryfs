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

    template<class Cipher>
    DerivedKey<Cipher::EncryptionKey::BINARY_LENGTH> CryConfigEncryptorFactory::_loadKey(cpputils::Deserializer *deserializer,
                                                                                         const std::string &password) {
        auto keyConfig = DerivedKeyConfig::load(deserializer);
        //TODO This is only kept here to recognize when this is run in tests. After tests are faster, replace this with something in main(), saying something like "Loading configuration file..."
        std::cout << "Deriving secure key for config file..." << std::flush;
        auto key = SCrypt().generateKeyFromConfig<Cipher::EncryptionKey::BINARY_LENGTH>(password, keyConfig);
        std::cout << "done" << std::endl;
        return DerivedKey<Cipher::EncryptionKey::BINARY_LENGTH>(std::move(keyConfig), std::move(key));
    }

    optional <unique_ref<CryConfigEncryptor>> CryConfigEncryptorFactory::loadKey(const Data &ciphertext,
                                                                          const string &password) {
        Deserializer deserializer(&ciphertext);
        try {
            CryConfigEncryptor::checkHeader(&deserializer);
            auto key = _loadKey<blockstore::encrypted::AES256_GCM>(&deserializer, password); //TODO Allow other ciphers
            return optional < unique_ref < CryConfigEncryptor >> (make_unique_ref < ConcreteCryConfigEncryptor <
                                                                  blockstore::encrypted::AES256_GCM >>
                                                                  (std::move(key))); //TODO Allow other ciphers
        } catch (const std::exception &e) {
            LOG(ERROR) << "Error loading configuration: " << e.what();
            return none; // This can be caused by invalid loaded data and is not necessarily a programming logic error. Don't throw exception.
        }
    }

}