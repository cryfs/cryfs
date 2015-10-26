#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGENCRYPTOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGENCRYPTOR_H

#include <messmer/cpp-utils/data/Serializer.h>
#include <messmer/cpp-utils/data/Deserializer.h>
#include <messmer/cpp-utils/data/Data.h>
#include <messmer/cpp-utils/random/Random.h>
#include <messmer/cpp-utils/logging/logging.h>
#include <string>
#include <utility>
#include <stdexcept>
#include "crypto/Scrypt.h"
#include <sstream>

namespace cryfs {
    //TODO Test
    //TODO Don't only encrypt with the main cipher, but also use user specified cipher.
    //TODO Use own exception for cpputils::Serializer/cpputils::Deserializer errors and only catch them
    template<class Cipher>
    class CryConfigEncryptor {
    public:
        using ConfigEncryptionKey = DerivedKey<Cipher::EncryptionKey::BINARY_LENGTH>;
        static constexpr size_t CONFIG_SIZE = 1024;  // Config data is grown to this size before encryption to hide its actual size

        static ConfigEncryptionKey deriveKey(const std::string &password);
        static boost::optional<std::pair<ConfigEncryptionKey, cpputils::Data>> decrypt(const cpputils::Data &ciphertext, const std::string &password);
        static cpputils::Data encrypt(const cpputils::Data &plaintext, const ConfigEncryptionKey &key);
    private:
        static std::pair<ConfigEncryptionKey, cpputils::Data> _decrypt(cpputils::Deserializer *deserializer, const std::string &password);
        static void _checkHeader(cpputils::Deserializer *deserializer);
        static ConfigEncryptionKey _loadKey(cpputils::Deserializer *deserializer, const std::string &password);
        static cpputils::Data _loadAndDecryptConfigData(cpputils::Deserializer *deserializer, const typename Cipher::EncryptionKey &key);
        static cpputils::Data _serialize(const cpputils::Data &ciphertext, const ConfigEncryptionKey &key);
        //TODO Test that encrypted config data always has the same size, no matter how big the plaintext config data
        static cpputils::Data _addPadding(const cpputils::Data &data);
        static boost::optional<cpputils::Data> _removePadding(const cpputils::Data &data);

        static const std::string HEADER;
    };

    template<class Cipher> const std::string CryConfigEncryptor<Cipher>::HEADER = "cryfs.config;0.8.1;scrypt";

    template<class Cipher>
    typename CryConfigEncryptor<Cipher>::ConfigEncryptionKey CryConfigEncryptor<Cipher>::deriveKey(const std::string &password) {
        //TODO This is only kept here to recognize when this is run in tests. After tests are faster, replace this with something in main(), saying something like "Loading configuration file..."
        std::cout << "Deriving secure key for config file..." << std::flush;
        auto key = SCrypt().generateKey<Cipher::EncryptionKey::BINARY_LENGTH>(password);
        std::cout << "done" << std::endl;
        return key;
    }

    template<class Cipher>
    boost::optional<std::pair<typename CryConfigEncryptor<Cipher>::ConfigEncryptionKey, cpputils::Data>>
    CryConfigEncryptor<Cipher>::decrypt(const cpputils::Data &data, const std::string &password) {
        cpputils::Deserializer deserializer(&data);
        try {
            auto result = _decrypt(&deserializer, password);
            deserializer.finished();
            return result;
        } catch (const std::exception &e) {
            cpputils::logging::LOG(cpputils::logging::ERROR) << "Error loading configuration: " << e.what();
            return boost::none; // This can be caused by invalid loaded data and is not necessarily a programming logic error. Don't throw exception.
        }
    };

    template<class Cipher>
    std::pair<typename CryConfigEncryptor<Cipher>::ConfigEncryptionKey, cpputils::Data>
    CryConfigEncryptor<Cipher>::_decrypt(cpputils::Deserializer *deserializer, const std::string &password) {
        _checkHeader(deserializer);
        auto key = _loadKey(deserializer, password);
        auto configData = _loadAndDecryptConfigData(deserializer, key.key());

        return std::make_pair(std::move(key), std::move(configData));
    };

    template<class Cipher>
    void CryConfigEncryptor<Cipher>::_checkHeader(cpputils::Deserializer *deserializer) {
        std::string header = deserializer->readString();
        if (header != HEADER) {
            throw std::runtime_error("Invalid header");
        }
    }

    template<class Cipher>
    typename CryConfigEncryptor<Cipher>::ConfigEncryptionKey CryConfigEncryptor<Cipher>::_loadKey(cpputils::Deserializer *deserializer, const std::string &password) {
        auto keyConfig = DerivedKeyConfig::load(deserializer);
        //TODO This is only kept here to recognize when this is run in tests. After tests are faster, replace this with something in main(), saying something like "Loading configuration file..."
        std::cout << "Deriving secure key for config file..." << std::flush;
        auto key = SCrypt().generateKeyFromConfig<Cipher::EncryptionKey::BINARY_LENGTH>(password, keyConfig);
        std::cout << "done" << std::endl;
        return ConfigEncryptionKey(std::move(keyConfig), std::move(key));
    }

    template<class Cipher>
    cpputils::Data CryConfigEncryptor<Cipher>::_loadAndDecryptConfigData(cpputils::Deserializer *deserializer, const typename Cipher::EncryptionKey &key) {
        auto ciphertext = deserializer->readData();
        auto decrypted = Cipher::decrypt(static_cast<const uint8_t*>(ciphertext.data()), ciphertext.size(), key);
        if (decrypted == boost::none) {
            throw std::runtime_error("Couldn't decrypt config file. Wrong password?");
        }
        auto configData = _removePadding(*decrypted);
        if (configData == boost::none) {
            throw std::runtime_error("Couldn't decrypt config file because of wrong padding");
        }
        return std::move(*configData);
    }

    template<class Cipher>
    cpputils::Data CryConfigEncryptor<Cipher>::encrypt(const cpputils::Data &plaintext, const ConfigEncryptionKey &key) {
        auto paddedPlaintext = _addPadding(plaintext);
        auto ciphertext = Cipher::encrypt(static_cast<const uint8_t*>(paddedPlaintext.data()), paddedPlaintext.size(), key.key());
        return _serialize(ciphertext, key);
    }

    template <class Cipher>
    cpputils::Data CryConfigEncryptor<Cipher>::_serialize(const cpputils::Data &ciphertext, const ConfigEncryptionKey &key) {
        try {
            cpputils::Serializer serializer(cpputils::Serializer::StringSize(HEADER)
                                            + key.config().serializedSize()
                                            + cpputils::Serializer::DataSize(ciphertext));
            serializer.writeString(HEADER);
            key.config().serialize(&serializer);
            serializer.writeData(ciphertext);
            return serializer.finished();
        } catch (const std::exception &e) {
            cpputils::logging::LOG(cpputils::logging::ERROR) << "Error serializing CryConfigEncryptor: " << e.what();
            throw; // This is a programming logic error. Pass through exception.
        }
    }

    template<class Cipher>
    cpputils::Data CryConfigEncryptor<Cipher>::_addPadding(const cpputils::Data &data) {
        uint32_t size = data.size();
        ASSERT(size < CONFIG_SIZE - sizeof(size), "Config data too large. We should increase CONFIG_SIZE.");
        cpputils::Data randomData = cpputils::Random::PseudoRandom().get(CONFIG_SIZE-sizeof(size)-size);
        ASSERT(sizeof(size) + size + randomData.size() == CONFIG_SIZE, "Calculated size of randomData incorrectly");
        cpputils::Data result(CONFIG_SIZE);
        std::memcpy(reinterpret_cast<char*>(result.data()), &size, sizeof(size));
        std::memcpy(reinterpret_cast<char*>(result.dataOffset(sizeof(size))), reinterpret_cast<const char*>(data.data()), size);
        std::memcpy(reinterpret_cast<char*>(result.dataOffset(sizeof(size)+size)), reinterpret_cast<const char*>(randomData.data()), randomData.size());
        return result;
    }

    template<class Cipher>
    boost::optional<cpputils::Data> CryConfigEncryptor<Cipher>::_removePadding(const cpputils::Data &data) {
        uint32_t size;
        std::memcpy(&size, reinterpret_cast<const char*>(data.data()), sizeof(size));
        if(sizeof(size) + size >= data.size()) {
            cpputils::logging::LOG(cpputils::logging::ERROR) << "Config file is invalid: Invalid padding.";
            return boost::none;
        };
        cpputils::Data result(size);
        std::memcpy(reinterpret_cast<char*>(result.data()), reinterpret_cast<const char*>(data.dataOffset(sizeof(size))), size);
        return std::move(result);
    }
}

#endif
