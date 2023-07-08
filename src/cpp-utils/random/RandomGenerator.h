#ifndef MESSMER_CPPUTILS_RANDOM_RANDOMGENERATOR_H
#define MESSMER_CPPUTILS_RANDOM_RANDOMGENERATOR_H

#include "../data/FixedSizeData.h"
#include "../data/Data.h"

namespace cpputils {
    class RandomGenerator {
    public:
        RandomGenerator();
        virtual ~RandomGenerator() = default;

        template<size_t SIZE> FixedSizeData<SIZE> getFixedSize();
        Data get(size_t size);

        void write(void *target, size_t size);

    protected:
        virtual void _get(void *target, size_t bytes) = 0;
    private:
        static std::mutex _mutex;

        DISALLOW_COPY_AND_ASSIGN(RandomGenerator);
    };

    inline RandomGenerator::RandomGenerator() {
    }

    inline void RandomGenerator::write(void *target, size_t size) {
        _get(target, size);
    }

    template<size_t SIZE> inline FixedSizeData<SIZE> RandomGenerator::getFixedSize() {
        FixedSizeData<SIZE> result = FixedSizeData<SIZE>::Null();
        _get(result.data(), SIZE);
        return result;
    }

    inline Data RandomGenerator::get(size_t size) {
        Data result(size);
        _get(result.data(), size);
        return result;
    }
}

#endif
