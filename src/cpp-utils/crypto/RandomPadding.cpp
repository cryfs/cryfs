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
        std::memcpy(reinterpret_cast<char*>(result.data()), &size, sizeof(size));
        std::memcpy(reinterpret_cast<char*>(result.dataOffset(sizeof(size))), reinterpret_cast<const char*>(data.data()), size);
        std::memcpy(reinterpret_cast<char*>(result.dataOffset(sizeof(size)+size)), reinterpret_cast<const char*>(randomData.data()), randomData.size());
        return result;
    }

    optional<Data> RandomPadding::remove(const Data &data) {
        uint32_t size;
        std::memcpy(&size, reinterpret_cast<const char*>(data.data()), sizeof(size));
        if(sizeof(size) + size >= data.size()) {
            LOG(ERROR) << "Config file is invalid: Invalid padding.";
            return boost::none;
        };
        Data result(size);
        std::memcpy(reinterpret_cast<char*>(result.data()), reinterpret_cast<const char*>(data.dataOffset(sizeof(size))), size);
        return std::move(result);
    }
}
