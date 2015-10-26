#include "CryConfigEncryptor.h"
#include <messmer/cpp-utils/logging/logging.h>

using std::string;
using cpputils::Deserializer;
using cpputils::Serializer;

namespace cryfs {
    const string CryConfigEncryptor::HEADER = "cryfs.config;0.8.1;scrypt";

    void CryConfigEncryptor::checkHeader(Deserializer *deserializer) {
        string header = deserializer->readString();
        if (header != HEADER) {
            throw std::runtime_error("Invalid header");
        }
    }

    void CryConfigEncryptor::writeHeader(Serializer *serializer) {
        serializer->writeString(HEADER);
    }

}