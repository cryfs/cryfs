#include "CryConfigEncryptor.h"

using std::string;
using cpputils::Deserializer;
using cpputils::Serializer;
using cpputils::unique_ref;
using cpputils::Data;
using boost::optional;
using boost::none;

namespace cryfs {
    const string CryConfigEncryptor::HEADER = "cryfs.config;0.8.1;scrypt";

    CryConfigEncryptor::CryConfigEncryptor(unique_ref<InnerEncryptor> innerEncryptor, DerivedKeyConfig keyConfig)
            : _innerEncryptor(std::move(innerEncryptor)), _keyConfig(std::move(keyConfig)) {
    }

    void CryConfigEncryptor::checkHeader(Deserializer *deserializer) {
        string header = deserializer->readString();
        if (header != HEADER) {
            throw std::runtime_error("Invalid header");
        }
    }

    void CryConfigEncryptor::writeHeader(Serializer *serializer) {
        serializer->writeString(HEADER);
    }

    Data CryConfigEncryptor::encrypt(const Data &plaintext) {
        auto ciphertext = _innerEncryptor->encrypt(plaintext);
        return _serialize(ciphertext);
    }

    Data CryConfigEncryptor::_serialize(const Data &ciphertext) {
        try {
            Serializer serializer(Serializer::StringSize(HEADER)
                                  + _keyConfig.serializedSize()
                                  + Serializer::DataSize(ciphertext));
            writeHeader(&serializer);
            _keyConfig.serialize(&serializer);
            serializer.writeData(ciphertext);
            return serializer.finished();
        } catch (const std::exception &e) {
            cpputils::logging::LOG(cpputils::logging::ERROR) << "Error serializing CryConfigEncryptor: " << e.what();
            throw; // This is a programming logic error. Pass through exception.
        }
    }

    optional<Data> CryConfigEncryptor::decrypt(const Data &plaintext) {
        Deserializer deserializer(&plaintext);
        try {
            checkHeader(&deserializer);
            _ignoreKey(&deserializer);
            auto configData = _loadAndDecryptConfigData(&deserializer);
            deserializer.finished();
            return configData;
        } catch (const std::exception &e) {
            cpputils::logging::LOG(cpputils::logging::ERROR) << "Error loading configuration: " << e.what();
            return boost::none; // This can be caused by invalid loaded data and is not necessarily a programming logic error. Don't throw exception.
        }
    }

    void CryConfigEncryptor::_ignoreKey(Deserializer *deserializer) {
        DerivedKeyConfig::load(deserializer);
    }

    optional<Data> CryConfigEncryptor::_loadAndDecryptConfigData(Deserializer *deserializer) {
        auto ciphertext = deserializer->readData();
        return _innerEncryptor->decrypt(ciphertext);
    }
}
