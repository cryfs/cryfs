#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CONCRETECRYCONFIGENCRYPTOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CONCRETECRYCONFIGENCRYPTOR_H

#include <messmer/cpp-utils/data/Serializer.h>
#include <messmer/cpp-utils/data/Deserializer.h>
#include "RandomPadding.h"
#include "InnerEncryptor.h"
#include "kdf/DerivedKey.h"

namespace cryfs {
    //TODO Test
    template<class Cipher>
    class ConcreteInnerEncryptor: public InnerEncryptor {
    public:
        static constexpr size_t CONFIG_SIZE = 512;  // Inner config data is grown to this size before encryption to hide its actual size

        ConcreteInnerEncryptor(typename Cipher::EncryptionKey key, const std::string &cipherName);

        cpputils::Data encrypt(const cpputils::Data &plaintext) const override;
        boost::optional<cpputils::Data> decrypt(const cpputils::Data &ciphertext) const override;
    private:

        cpputils::Data _serialize(const cpputils::Data &data) const;
        boost::optional<cpputils::Data> _deserialize(const cpputils::Data &data) const;

        std::string _cipherName;
        typename Cipher::EncryptionKey _key;
    };

    template<class Cipher>
    ConcreteInnerEncryptor<Cipher>::ConcreteInnerEncryptor(typename Cipher::EncryptionKey key, const std::string &cipherName)
            :  _cipherName(cipherName), _key(std::move(key)) {
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
        auto configData = RandomPadding::remove(*decrypted);
        if (configData == boost::none) {
            return boost::none;
        }
        return std::move(*configData);
    }

    template<class Cipher>
    boost::optional<cpputils::Data> ConcreteInnerEncryptor<Cipher>::_deserialize(const cpputils::Data &ciphertext) const {
        cpputils::Deserializer deserializer(&ciphertext);
        try {
            std::string readCipherName = deserializer.readString();
            if (readCipherName != _cipherName) {
                cpputils::logging::LOG(cpputils::logging::ERROR) << "Wrong inner cipher used";
                return boost::none;
            }
            auto result = deserializer.readData();
            deserializer.finished();
            return result;
        } catch (const std::exception &e) {
            cpputils::logging::LOG(cpputils::logging::ERROR) << "Error serializing inner configuration: " << e.what();
            return boost::none; // This can be caused by invalid input data and does not have to be a programming error. Don't throw exception.
        }
    }

    template<class Cipher>
    cpputils::Data ConcreteInnerEncryptor<Cipher>::encrypt(const cpputils::Data &plaintext) const {
        auto paddedPlaintext = RandomPadding::add(plaintext, CONFIG_SIZE);
        auto encrypted = Cipher::encrypt(static_cast<const uint8_t*>(paddedPlaintext.data()), paddedPlaintext.size(), _key);
        return _serialize(encrypted);
    }

    template<class Cipher>
    cpputils::Data ConcreteInnerEncryptor<Cipher>::_serialize(const cpputils::Data &ciphertext) const {
        try {
            cpputils::Serializer serializer(cpputils::Serializer::StringSize(_cipherName)
                                            + cpputils::Serializer::DataSize(ciphertext));
            serializer.writeString(_cipherName);
            serializer.writeData(ciphertext);
            return serializer.finished();
        } catch (const std::exception &e) {
            cpputils::logging::LOG(cpputils::logging::ERROR) << "Error serializing inner configuration: " << e.what();
            throw; // This is a programming logic error, pass through exception.
        }
    }
}

#endif
