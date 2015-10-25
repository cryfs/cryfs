#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGENCRYPTOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGENCRYPTOR_H

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

        //TODO Test that encrypted config data always has the same size, no matter how big the plaintext config data
        static cpputils::Data _addPadding(const cpputils::Data &data);
        static boost::optional<cpputils::Data> _removePadding(const cpputils::Data &data);

        static ConfigEncryptionKey deriveKey(const std::string &password);
        static boost::optional<std::pair<ConfigEncryptionKey, cpputils::Data>> decrypt(const cpputils::Data &ciphertext, const std::string &password);
        static cpputils::Data encrypt(const cpputils::Data &plaintext, const ConfigEncryptionKey &key);
    private:

        static boost::optional<std::pair<ConfigEncryptionKey, cpputils::Data>> _decrypt(std::stringstream &stream, const std::string &password);
        static bool _checkHeader(std::stringstream &stream);
        static boost::optional<ConfigEncryptionKey> _loadKey(std::stringstream &stream, const std::string &password);
        static std::streampos _getStreamSize(std::istream &stream);
        static boost::optional<cpputils::Data> _loadAndDecryptData(std::stringstream &stream, const typename Cipher::EncryptionKey &key);

        static const std::string header;
    };

    template<class Cipher> const std::string CryConfigEncryptor<Cipher>::header = "scrypt";

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
    CryConfigEncryptor<Cipher>::decrypt(const cpputils::Data &ciphertext, const std::string &password) {
        std::stringstream stream;
        ciphertext.StoreToStream(stream);
        return _decrypt(stream, password);
    };

    template<class Cipher>
    boost::optional<std::pair<typename CryConfigEncryptor<Cipher>::ConfigEncryptionKey, cpputils::Data>>
    CryConfigEncryptor<Cipher>::_decrypt(std::stringstream &stream, const std::string &password) {
        if(!_checkHeader(stream)) {
            return boost::none;
        }
        auto key = _loadKey(stream, password);
        if (key == boost::none) {
            return boost::none;
        }

        auto plaintext = _loadAndDecryptData(stream, key->key());
        if (plaintext == boost::none) {
            return boost::none;
        }
        return std::make_pair(std::move(*key), std::move(*plaintext));
    };

    template<class Cipher>
    bool CryConfigEncryptor<Cipher>::_checkHeader(std::stringstream &stream) {
        char readHeader[header.size()+1];
        stream.read(readHeader, header.size()+1);
        bool headerOk = readHeader[header.size()] == '\0' && header == std::string(readHeader);
        if (!headerOk) {
            cpputils::logging::LOG(cpputils::logging::ERROR) << "Wrong config file format";
        }
        return headerOk;
    }

    template<class Cipher>
    boost::optional<typename CryConfigEncryptor<Cipher>::ConfigEncryptionKey> CryConfigEncryptor<Cipher>::_loadKey(std::stringstream &stream, const std::string &password) {
        uint32_t serializedKeyConfigSize;
        uint32_t streamSize = _getStreamSize(stream);
        if(streamSize < sizeof(serializedKeyConfigSize)) {
            return boost::none;
        }
        stream.read(reinterpret_cast<char*>(&serializedKeyConfigSize), sizeof(serializedKeyConfigSize));
        if((streamSize-sizeof(serializedKeyConfigSize)) < serializedKeyConfigSize) {
            return boost::none;
        }
        cpputils::Data serializedKeyConfig = cpputils::Data::LoadFromStream(stream, serializedKeyConfigSize);
        auto keyConfig = DerivedKeyConfig::load(serializedKeyConfig);
        if (keyConfig == boost::none) {
            return boost::none;
        }
        std::cout << "Deriving secure key for config file..." << std::flush;
        auto key = SCrypt().generateKeyFromConfig<Cipher::EncryptionKey::BINARY_LENGTH>(password, *keyConfig);
        std::cout << "done" << std::endl;
        return ConfigEncryptionKey(std::move(*keyConfig), std::move(key));
    }

    template<class Cipher>
    std::streampos CryConfigEncryptor<Cipher>::_getStreamSize(std::istream &stream) {
        auto current_pos = stream.tellg();

        //Retrieve length
        stream.seekg(0, stream.end);
        auto endpos = stream.tellg();

        //Restore old position
        stream.seekg(current_pos, stream.beg);

        return endpos - current_pos;
    }

    template<class Cipher>
    boost::optional<cpputils::Data> CryConfigEncryptor<Cipher>::_loadAndDecryptData(std::stringstream &stream, const typename Cipher::EncryptionKey &key) {
        auto data = cpputils::Data::LoadFromStream(stream);
        auto decrypted = Cipher::decrypt(static_cast<const uint8_t*>(data.data()), data.size(), key);
        if (decrypted == boost::none) {
            cpputils::logging::LOG(cpputils::logging::ERROR) <<  "Couldn't load config file. Wrong password?";
            return boost::none;
        }
        return _removePadding(*decrypted);
    }

    template<class Cipher>
    cpputils::Data CryConfigEncryptor<Cipher>::encrypt(const cpputils::Data &plaintext, const ConfigEncryptionKey &key) {
        std::stringstream stream;
        stream.write(header.c_str(), header.size()+1);
        cpputils::Data serializedKeyConfig = key.config().save();
        uint32_t serializedKeyConfigSize = serializedKeyConfig.size();
        stream.write(reinterpret_cast<const char*>(&serializedKeyConfigSize), sizeof(serializedKeyConfigSize));
        serializedKeyConfig.StoreToStream(stream);
        auto paddedPlaintext = _addPadding(plaintext);
        auto ciphertext = Cipher::encrypt(static_cast<const uint8_t*>(paddedPlaintext.data()), paddedPlaintext.size(), key.key());
        ciphertext.StoreToStream(stream);
        return cpputils::Data::LoadFromStream(stream);
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
