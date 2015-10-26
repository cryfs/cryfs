#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CONCRETECRYCONFIGENCRYPTOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CONCRETECRYCONFIGENCRYPTOR_H

#include <messmer/cpp-utils/data/Serializer.h>
#include <messmer/cpp-utils/data/Deserializer.h>
#include "RandomPadding.h"
#include "CryConfigEncryptor.h"

namespace cryfs {
    //TODO Test
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
    ConcreteCryConfigEncryptor<Cipher>::ConcreteCryConfigEncryptor(ConfigEncryptionKey key): _key(std::move(key)) {
    }

    template<class Cipher>
    boost::optional<cpputils::Data> ConcreteCryConfigEncryptor<Cipher>::decrypt(const cpputils::Data &data) {
        cpputils::Deserializer deserializer(&data);
        try {
            checkHeader(&deserializer);
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
            writeHeader(&serializer);
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
