#include "InnerConfig.h"
#include <cpp-utils/logging/logging.h>

using std::string;
using std::exception;
using cpputils::Deserializer;
using cpputils::Serializer;
using cpputils::Data;
using boost::optional;
using boost::none;
using namespace cpputils::logging;

namespace cryfs {
    const string InnerConfig::HEADER = "cryfs.config.inner;0";

    Data InnerConfig::serialize() const {
        try {
            Serializer serializer(Serializer::StringSize(HEADER)
                                  + Serializer::StringSize(cipherName)
                                  + encryptedConfig.size());
            serializer.writeString(HEADER);
            serializer.writeString(cipherName);
            serializer.writeTailData(encryptedConfig);
            return serializer.finished();
        } catch (const exception &e) {
            LOG(ERR, "Error serializing inner configuration: {}", e.what());
            throw; // This is a programming logic error, pass through exception.
        }
    }

    optional<InnerConfig> InnerConfig::deserialize(const Data &data) {
        Deserializer deserializer(&data);
        try {
            _checkHeader(&deserializer);
            string cipherName = deserializer.readString();
            auto result = deserializer.readTailData();
            deserializer.finished();
            return InnerConfig {cipherName, std::move(result)};
        } catch (const exception &e) {
            LOG(ERR, "Error deserializing inner configuration: {}", e.what());
            return none; // This can be caused by invalid input data and does not have to be a programming error. Don't throw exception.
        }
    }

    void InnerConfig::_checkHeader(Deserializer *deserializer) {
        string header = deserializer->readString();
        if (header != HEADER) {
            throw std::runtime_error("Invalid header. Maybe this filesystem was created with a different version of CryFS?");
        }
    }

    void InnerConfig::_writeHeader(Serializer *serializer) {
        serializer->writeString(HEADER);
    }
}
