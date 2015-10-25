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
    //TODO Refactor. Functions too large.
    template<class Cipher>
    class CryConfigEncryptor {
    public:
        using ConfigEncryptionKey = DerivedKey<Cipher::EncryptionKey::BINARY_LENGTH>;
        static constexpr size_t CONFIG_SIZE = 1024;  // Config data is grown to this size before encryption to hide its actual size

        static ConfigEncryptionKey deriveKey(const std::string &password);
        static boost::optional<std::pair<ConfigEncryptionKey, cpputils::Data>> decrypt(const cpputils::Data &ciphertext, const std::string &password);
        static cpputils::Data encrypt(const cpputils::Data &plaintext, const ConfigEncryptionKey &key);
    private:
        static boost::optional<std::pair<ConfigEncryptionKey, cpputils::Data>> _decrypt(const std::string &header, const cpputils::Data &serializedKeyConfig, const cpputils::Data &ciphertext, const std::string &password);
        static bool _checkHeader(const std::string &header);
        static boost::optional<ConfigEncryptionKey> _loadKey(const cpputils::Data &serializedKeyConfig, const std::string &password);
        static boost::optional<cpputils::Data> _loadAndDecryptConfigData(const cpputils::Data &ciphertext, const typename Cipher::EncryptionKey &key);
        //TODO Test that encrypted config data always has the same size, no matter how big the plaintext config data
        static cpputils::Data _addPadding(const cpputils::Data &data);
        static boost::optional<cpputils::Data> _removePadding(const cpputils::Data &data);

        static const std::string HEADER;
    };

    template<class Cipher> const std::string CryConfigEncryptor<Cipher>::HEADER = "scrypt";

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
        try {
            cpputils::Deserializer deserializer(&data);
            std::string header = deserializer.readString();
            cpputils::Data serializedKeyConfig = deserializer.readData();
            cpputils::Data ciphertext = deserializer.readData();
            deserializer.finished();

            return _decrypt(header, serializedKeyConfig, ciphertext, password);

        } catch (const std::exception &e) {
            cpputils::logging::LOG(cpputils::logging::ERROR) << "Error deserializing CryConfigEncryptor: " << e.what();
            return boost::none; // This can be caused by bad loaded data and is not necessarily a programming logic error. Don't throw exception.
        }
    };

    template<class Cipher>
    boost::optional<std::pair<typename CryConfigEncryptor<Cipher>::ConfigEncryptionKey, cpputils::Data>>
    CryConfigEncryptor<Cipher>::_decrypt(const std::string &header, const cpputils::Data &serializedKeyConfig, const cpputils::Data &ciphertext, const std::string &password) {
        if (!_checkHeader(header)) {
            return boost::none;
        }

        auto key = _loadKey(serializedKeyConfig, password);
        if (key == boost::none) {
            return boost::none;
        }

        auto configData = _loadAndDecryptConfigData(ciphertext, key->key());
        if (configData == boost::none) {
            return boost::none;
        }

        return std::make_pair(std::move(*key), std::move(*configData));
    };

    template<class Cipher>
    bool CryConfigEncryptor<Cipher>::_checkHeader(const std::string &header) {
        if (header != HEADER) {
            cpputils::logging::LOG(cpputils::logging::ERROR) << "Error deserializing CryConfigEncryptor: Invalid header.";
            return false;
        }
        return true;
    }

    template<class Cipher>
    boost::optional<typename CryConfigEncryptor<Cipher>::ConfigEncryptionKey> CryConfigEncryptor<Cipher>::_loadKey(const cpputils::Data &serializedKeyConfig, const std::string &password) {
        auto keyConfig = DerivedKeyConfig::load(serializedKeyConfig);
        if (keyConfig == boost::none) {
            cpputils::logging::LOG(cpputils::logging::ERROR) << "Error deserializing CryConfigEncryptor: Invalid key configuration.";
            return boost::none;
        }
        //TODO This is only kept here to recognize when this is run in tests. After tests are faster, replace this with something in main(), saying something like "Loading configuration file..."
        std::cout << "Deriving secure key for config file..." << std::flush;
        auto key = SCrypt().generateKeyFromConfig<Cipher::EncryptionKey::BINARY_LENGTH>(password, *keyConfig);
        std::cout << "done" << std::endl;
        return ConfigEncryptionKey(std::move(*keyConfig), std::move(key));
    }

    template<class Cipher>
    boost::optional<cpputils::Data> CryConfigEncryptor<Cipher>::_loadAndDecryptConfigData(const cpputils::Data &ciphertext, const typename Cipher::EncryptionKey &key) {
        auto decrypted = Cipher::decrypt(static_cast<const uint8_t*>(ciphertext.data()), ciphertext.size(), key);
        if (decrypted == boost::none) {
            cpputils::logging::LOG(cpputils::logging::ERROR) <<  "Couldn't decrypt config file. Wrong password?";
            return boost::none;
        }
        auto configData = _removePadding(*decrypted);
        if (configData == boost::none) {
            cpputils::logging::LOG(cpputils::logging::ERROR) <<  "Couldn't decrypt config file because of wrong padding.";
            return boost::none;
        }
        return configData;
    }

    template<class Cipher>
    cpputils::Data CryConfigEncryptor<Cipher>::encrypt(const cpputils::Data &plaintext, const ConfigEncryptionKey &key) {
        cpputils::Data serializedKeyConfig = key.config().save();
        auto paddedPlaintext = _addPadding(plaintext);
        auto ciphertext = Cipher::encrypt(static_cast<const uint8_t*>(paddedPlaintext.data()), paddedPlaintext.size(), key.key());
        try {
            cpputils::Serializer serializer(cpputils::Serializer::StringSize(HEADER)
                                            + cpputils::Serializer::DataSize(serializedKeyConfig)
                                            + cpputils::Serializer::DataSize(ciphertext));
            serializer.writeString(HEADER);
            serializer.writeData(serializedKeyConfig);
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
        if(sizeof(size) + size >= CONFIG_SIZE) {
            cpputils::logging::LOG(cpputils::logging::ERROR) << "Config file is invalid: Invalid padding.";
            return boost::none;
        };
        cpputils::Data result(size);
        std::memcpy(reinterpret_cast<char*>(result.data()), reinterpret_cast<const char*>(data.dataOffset(sizeof(size))), size);
        return std::move(result);
    }
}

#endif
