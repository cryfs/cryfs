#include "OuterConfig.h"

using std::string;
using std::exception;
using cpputils::Data;
using cpputils::Serializer;
using cpputils::Deserializer;
using cpputils::DerivedKeyConfig;
using boost::optional;
using boost::none;
using namespace cpputils::logging;

namespace cryfs {
    const string OuterConfig::HEADER = "cryfs.config;0;scrypt";

    void OuterConfig::_checkHeader(Deserializer *deserializer) {
        string header = deserializer->readString();
        if (header != HEADER) {
            throw std::runtime_error("Invalid header");
        }
    }

    void OuterConfig::_writeHeader(Serializer *serializer) {
        serializer->writeString(HEADER);
    }

    Data OuterConfig::serialize() const {
        try {
            Serializer serializer(Serializer::StringSize(HEADER)
                                  + keyConfig.serializedSize()
                                  + encryptedInnerConfig.size());
            _writeHeader(&serializer);
            keyConfig.serialize(&serializer);
            serializer.writeTailData(encryptedInnerConfig);
            return serializer.finished();
        } catch (const exception &e) {
            LOG(ERROR) << "Error serializing CryConfigEncryptor: " << e.what();
            throw; // This is a programming logic error. Pass through exception.
        }
    }

    optional<OuterConfig> OuterConfig::deserialize(const Data &data) {
        Deserializer deserializer(&data);
        try {
            _checkHeader(&deserializer);
            auto keyConfig = DerivedKeyConfig::deserialize(&deserializer);
            auto encryptedInnerConfig = deserializer.readTailData();
            deserializer.finished();
            return OuterConfig {std::move(keyConfig), std::move(encryptedInnerConfig)};
        } catch (const exception &e) {
            LOG(ERROR) << "Error deserializing outer configuration: " << e.what();
            return none; // This can be caused by invalid input data and does not have to be a programming error. Don't throw exception.
        }
    }
}
