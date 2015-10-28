#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CONCRETECRYCONFIGENCRYPTOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CONCRETECRYCONFIGENCRYPTOR_H

#include <messmer/cpp-utils/data/Serializer.h>
#include <messmer/cpp-utils/data/Deserializer.h>
#include "InnerEncryptor.h"
#include <messmer/cpp-utils/crypto/RandomPadding.h>
#include <messmer/cpp-utils/crypto/kdf/DerivedKey.h>

namespace cryfs {
    //TODO Test
    template<class Cipher>
    class ConcreteInnerEncryptor: public InnerEncryptor {
    public:
        static constexpr size_t CONFIG_SIZE = 512;  // Inner config data is grown to this size before encryption to hide its actual size

        ConcreteInnerEncryptor(typename Cipher::EncryptionKey key);

        cpputils::Data encrypt(const cpputils::Data &plaintext) const override;
        boost::optional<cpputils::Data> decrypt(const cpputils::Data &ciphertext) const override;
    private:

        cpputils::Data _serialize(const cpputils::Data &data) const;
        boost::optional<cpputils::Data> _deserialize(const cpputils::Data &data) const;

        typename Cipher::EncryptionKey _key;
    };

    template<class Cipher>
    ConcreteInnerEncryptor<Cipher>::ConcreteInnerEncryptor(typename Cipher::EncryptionKey key)
            : _key(std::move(key)) {
    }

    template<class Cipher>
    boost::optional<cpputils::Data> ConcreteInnerEncryptor<Cipher>::decrypt(const cpputils::Data &ciphertext) const {
        auto data = _deserialize(ciphertext);
        if (data == boost::none) {
            return boost::none;
        }
        auto decrypted = Cipher::decrypt(static_cast<const uint8_t*>(data->data()), data->size(), _key);
        if (decrypted == boost::none) {
            return boost::none;
        }
        auto configData = cpputils::RandomPadding::remove(*decrypted);
        if (configData == boost::none) {
            return boost::none;
        }
        return std::move(*configData);
    }

    template<class Cipher>
    boost::optional<cpputils::Data> ConcreteInnerEncryptor<Cipher>::_deserialize(const cpputils::Data &ciphertext) const {
        cpputils::Deserializer deserializer(&ciphertext);
        try {
            _checkHeader(&deserializer);
            std::string readCipherName = deserializer.readString();
            if (readCipherName != Cipher::NAME) {
                cpputils::logging::LOG(cpputils::logging::ERROR) << "Wrong inner cipher used";
                return boost::none;
            }
            auto result = deserializer.readTailData();
            deserializer.finished();
            return std::move(result); // TODO This std::move() is not necessary on newer gcc versions. Remove it and look for other occurrences of the same.
        } catch (const std::exception &e) {
            cpputils::logging::LOG(cpputils::logging::ERROR) << "Error serializing inner configuration: " << e.what();
            return boost::none; // This can be caused by invalid input data and does not have to be a programming error. Don't throw exception.
        }
    }

    template<class Cipher>
    cpputils::Data ConcreteInnerEncryptor<Cipher>::encrypt(const cpputils::Data &plaintext) const {
        auto paddedPlaintext = cpputils::RandomPadding::add(plaintext, CONFIG_SIZE);
        auto encrypted = Cipher::encrypt(static_cast<const uint8_t*>(paddedPlaintext.data()), paddedPlaintext.size(), _key);
        return _serialize(encrypted);
    }

    template<class Cipher>
    cpputils::Data ConcreteInnerEncryptor<Cipher>::_serialize(const cpputils::Data &ciphertext) const {
        try {
            cpputils::Serializer serializer(cpputils::Serializer::StringSize(HEADER)
                                            + cpputils::Serializer::StringSize(Cipher::NAME)
                                            + ciphertext.size());
            serializer.writeString(HEADER);
            serializer.writeString(Cipher::NAME);
            serializer.writeTailData(ciphertext);
            return serializer.finished();
        } catch (const std::exception &e) {
            cpputils::logging::LOG(cpputils::logging::ERROR) << "Error serializing inner configuration: " << e.what();
            throw; // This is a programming logic error, pass through exception.
        }
    }
}

#endif
