#include "InnerEncryptor.h"

using std::string;
using cpputils::Deserializer;
using cpputils::Serializer;

namespace cryfs {
    const string InnerEncryptor::HEADER = "cryfs.config.inner;0";

    void InnerEncryptor::_checkHeader(Deserializer *deserializer) {
        string header = deserializer->readString();
        if (header != HEADER) {
            throw std::runtime_error("Invalid header");
        }
    }

    void InnerEncryptor::_writeHeader(Serializer *serializer) {
        serializer->writeString(HEADER);
    }

}