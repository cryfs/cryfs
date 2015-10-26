#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGENCRYPTOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGENCRYPTOR_H

#include <messmer/cpp-utils/data/Serializer.h>
#include <messmer/cpp-utils/data/Deserializer.h>
#include <messmer/cpp-utils/data/Data.h>
#include <messmer/cpp-utils/random/Random.h>
#include <messmer/cpp-utils/logging/logging.h>
#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <messmer/blockstore/implementations/encrypted/ciphers/ciphers.h>
#include <string>
#include <utility>
#include <stdexcept>
#include "crypto/Scrypt.h"
#include "RandomPadding.h"
#include <sstream>

namespace cryfs {
    //TODO Test
    //TODO Test that encrypted config data always has the same size, no matter how big the plaintext config data
    //TODO Don't only encrypt with the main cipher, but also use user specified cipher.
    //TODO Use own exception for cpputils::Serializer/cpputils::Deserializer errors and only catch them
    class CryConfigEncryptor {
    public:
        template<class Cipher> static cpputils::unique_ref<CryConfigEncryptor> deriveKey(const std::string &password);
        static boost::optional<cpputils::unique_ref<CryConfigEncryptor>> loadKey(const cpputils::Data &ciphertext, const std::string &password);

        virtual cpputils::Data encrypt(const cpputils::Data &plaintext) = 0;
        virtual boost::optional<cpputils::Data> decrypt(const cpputils::Data &plaintext) = 0;

    protected:
        static void _checkHeader(cpputils::Deserializer *deserializer);

    private:
        template<class Cipher> static DerivedKey<Cipher::EncryptionKey::BINARY_LENGTH> _loadKey(cpputils::Deserializer *deserializer, const std::string &password);

        static const std::string HEADER;
    };

    template<class Cipher>
    class ConcreteCryConfigEncryptor: public CryConfigEncryptor {
    public:
        using ConfigEncryptionKey = DerivedKey<Cipher::EncryptionKey::BINARY_LENGTH>;
        static constexpr size_t CONFIG_SIZE = 1024;  // Config data is grown to this size before encryption to hide its actual size

        ConcreteCryConfigEncryptor(ConfigEncryptionKey key);

        cpputils::Data encrypt(const cpputils::Data &plaintext) override;
        boost::optional<cpputils::Data> decrypt(const cpputils::Data &ciphertext) override;
    private:
        void _ignoreKey(cpputils::Deserializer *deserializer);
        cpputils::Data _loadAndDecryptConfigData(cpputils::Deserializer *deserializer);
        cpputils::Data _serialize(const cpputils::Data &ciphertext);

        ConfigEncryptionKey _key;
    };

    template<class Cipher>
    cpputils::unique_ref<CryConfigEncryptor> CryConfigEncryptor::deriveKey(const std::string &password) {
        //TODO This is only kept here to recognize when this is run in tests. After tests are faster, replace this with something in main(), saying something like "Loading configuration file..."
        std::cout << "Deriving secure key for config file..." << std::flush;
        auto key = SCrypt().generateKey<Cipher::EncryptionKey::BINARY_LENGTH>(password);
        std::cout << "done" << std::endl;
        return cpputils::make_unique_ref<ConcreteCryConfigEncryptor<Cipher>>(std::move(key));
    }

    inline boost::optional<cpputils::unique_ref<CryConfigEncryptor>> CryConfigEncryptor::loadKey(const cpputils::Data &ciphertext, const std::string &password) {
        cpputils::Deserializer deserializer(&ciphertext);
        try {
            _checkHeader(&deserializer);
            auto key = _loadKey<blockstore::encrypted::AES256_GCM>(&deserializer, password); //TODO Allow other ciphers
            return boost::optional<cpputils::unique_ref<CryConfigEncryptor>>(cpputils::make_unique_ref<ConcreteCryConfigEncryptor<blockstore::encrypted::AES256_GCM>>(std::move(key))); //TODO Allow other ciphers
        } catch (const std::exception &e) {
            cpputils::logging::LOG(cpputils::logging::ERROR) << "Error loading configuration: " << e.what();
            return boost::none; // This can be caused by invalid loaded data and is not necessarily a programming logic error. Don't throw exception.
        }
    }

    inline void CryConfigEncryptor::_checkHeader(cpputils::Deserializer *deserializer) {
        std::string header = deserializer->readString();
        if (header != HEADER) {
            throw std::runtime_error("Invalid header");
        }
    }

    template<class Cipher>
    DerivedKey<Cipher::EncryptionKey::BINARY_LENGTH> CryConfigEncryptor::_loadKey(cpputils::Deserializer *deserializer, const std::string &password) {
        auto keyConfig = DerivedKeyConfig::load(deserializer);
        //TODO This is only kept here to recognize when this is run in tests. After tests are faster, replace this with something in main(), saying something like "Loading configuration file..."
        std::cout << "Deriving secure key for config file..." << std::flush;
        auto key = SCrypt().generateKeyFromConfig<Cipher::EncryptionKey::BINARY_LENGTH>(password, keyConfig);
        std::cout << "done" << std::endl;
        return DerivedKey<Cipher::EncryptionKey::BINARY_LENGTH>(std::move(keyConfig), std::move(key));
    }



    template<class Cipher>
    ConcreteCryConfigEncryptor<Cipher>::ConcreteCryConfigEncryptor(ConfigEncryptionKey key): _key(std::move(key)) {
    }

    template<class Cipher>
    boost::optional<cpputils::Data> ConcreteCryConfigEncryptor<Cipher>::decrypt(const cpputils::Data &data) {
        cpputils::Deserializer deserializer(&data);
        try {
            _checkHeader(&deserializer);
            _ignoreKey(&deserializer);
            return _loadAndDecryptConfigData(&deserializer);
        } catch (const std::exception &e) {
            cpputils::logging::LOG(cpputils::logging::ERROR) << "Error loading configuration: " << e.what();
            return boost::none; // This can be caused by invalid loaded data and is not necessarily a programming logic error. Don't throw exception.
        }
    }

    template<class Cipher>
    void ConcreteCryConfigEncryptor<Cipher>::_ignoreKey(cpputils::Deserializer *deserializer) {
        DerivedKeyConfig::load(deserializer);
    }

    template<class Cipher>
    cpputils::Data ConcreteCryConfigEncryptor<Cipher>::_loadAndDecryptConfigData(cpputils::Deserializer *deserializer) {
        auto ciphertext = deserializer->readData();
        auto decrypted = Cipher::decrypt(static_cast<const uint8_t*>(ciphertext.data()), ciphertext.size(), _key.key());
        if (decrypted == boost::none) {
            throw std::runtime_error("Couldn't decrypt config file. Wrong password?");
        }
        auto configData = RandomPadding::remove(*decrypted);
        if (configData == boost::none) {
            throw std::runtime_error("Couldn't decrypt config file because of wrong padding");
        }
        return std::move(*configData);
    }

    template<class Cipher>
    cpputils::Data ConcreteCryConfigEncryptor<Cipher>::encrypt(const cpputils::Data &plaintext) {
        auto paddedPlaintext = RandomPadding::add(plaintext, CONFIG_SIZE);
        auto ciphertext = Cipher::encrypt(static_cast<const uint8_t*>(paddedPlaintext.data()), paddedPlaintext.size(), _key.key());
        return _serialize(ciphertext);
    }

    template <class Cipher>
    cpputils::Data ConcreteCryConfigEncryptor<Cipher>::_serialize(const cpputils::Data &ciphertext) {
        try {
            cpputils::Serializer serializer(cpputils::Serializer::StringSize(HEADER)
                                            + _key.config().serializedSize()
                                            + cpputils::Serializer::DataSize(ciphertext));
            serializer.writeString(HEADER);
            _key.config().serialize(&serializer);
            serializer.writeData(ciphertext);
            return serializer.finished();
        } catch (const std::exception &e) {
            cpputils::logging::LOG(cpputils::logging::ERROR) << "Error serializing CryConfigEncryptor: " << e.what();
            throw; // This is a programming logic error. Pass through exception.
        }
    }
}

#endif
