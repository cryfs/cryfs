#include <cpp-utils/data/SerializationHelper.h>
#include "RandomPadding.h"
#include "../logging/logging.h"
#include "../random/Random.h"

using boost::optional;
using namespace cpputils::logging;

namespace cpputils {
    Data RandomPadding::add(const Data &data, size_t targetSize) {
        uint32_t size = data.size();
        if (size >= targetSize - sizeof(size)) {
            throw std::runtime_error("Data too large. We should increase padding target size.");
        }
        Data randomData = Random::PseudoRandom().get(targetSize-sizeof(size)-size);
        ASSERT(sizeof(size) + size + randomData.size() == targetSize, "Calculated size of randomData incorrectly");
        Data result(targetSize);
        serialize<uint32_t>(result.data(), size);
        std::memcpy(result.dataOffset(sizeof(size)), data.data(), size);
        std::memcpy(result.dataOffset(sizeof(size)+size), randomData.data(), randomData.size());
        return result;
    }

    optional<Data> RandomPadding::remove(const Data &data) {
        uint32_t size = deserialize<uint32_t>(data.data());
        if(sizeof(size) + size >= data.size()) {
            LOG(ERR, "Config file is invalid: Invalid padding.");
            return boost::none;
        };
        Data result(size);
        std::memcpy(result.data(), data.dataOffset(sizeof(size)), size);
        return result;
    }
}
