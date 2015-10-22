#ifndef MESSMER_CPPUTILS_RANDOM_RANDOMGENERATOR_H
#define MESSMER_CPPUTILS_RANDOM_RANDOMGENERATOR_H

#include "../data/FixedSizeData.h"
#include "../data/Data.h"

namespace cpputils {
    class RandomGenerator {
    public:
        template<size_t SIZE> FixedSizeData<SIZE> getFixedSize();
        Data get(size_t size);

    protected:
        virtual void get(void *target, size_t bytes) = 0;
    private:
        static std::mutex _mutex;
    };

    template<size_t SIZE> inline FixedSizeData<SIZE> RandomGenerator::getFixedSize() {
        FixedSizeData<SIZE> result = FixedSizeData<SIZE>::Null();
        get(result.data(), SIZE);
        return result;
    }

    inline Data RandomGenerator::get(size_t size) {
        Data result(size);
        get(result.data(), size);
        return result;
    }
}

#endif
