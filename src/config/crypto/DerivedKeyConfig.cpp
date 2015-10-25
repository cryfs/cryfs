#include "DerivedKeyConfig.h"
#include <messmer/cpp-utils/assert/assert.h>
#include <messmer/cpp-utils/logging/logging.h>
#include <messmer/cpp-utils/data/Serializer.h>
#include <messmer/cpp-utils/data/Deserializer.h>

using std::istream;
using std::ostream;
using cpputils::Data;
using cpputils::Serializer;
using cpputils::Deserializer;
using boost::optional;
using boost::none;
using namespace cpputils::logging;

namespace cryfs {
    Data DerivedKeyConfig::save() const {
        Serializer serializer(Serializer::DataSize(_salt) + sizeof(uint64_t) + sizeof(uint32_t) + sizeof(uint32_t));
        try {
            serializer.writeData(_salt);
            serializer.writeUint64(_N);
            serializer.writeUint32(_r);
            serializer.writeUint32(_p);
            return serializer.finished();
        } catch (const std::exception &e) {
            LOG(ERROR) << "Error when trying to serialize DerivedKeyConfig: " << e.what();
            //This is a programming logic error. Pass-through exception.
            throw;
        }
    }

    boost::optional<DerivedKeyConfig> DerivedKeyConfig::load(const Data &data) {
        Deserializer deserializer(&data);
        try {
            Data salt = deserializer.readData();
            uint64_t N = deserializer.readUint64();
            uint32_t r = deserializer.readUint32();
            uint32_t p = deserializer.readUint32();
            deserializer.finished();
            return DerivedKeyConfig(std::move(salt), N, r, p);
        } catch (const std::exception &e) {
            LOG(ERROR) << "Error when trying to deserialize DerivedKeyConfig: " << e.what();
            //This might be caused by invalid data loaded and does not have to be a programming logic error. Don't throw exception.
            return none;
        }
    }
}
