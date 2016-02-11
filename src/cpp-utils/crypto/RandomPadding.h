#pragma once
#ifndef MESSMER_CPPUTILS_CRYPTO_RANDOMPADDING_H
#define MESSMER_CPPUTILS_CRYPTO_RANDOMPADDING_H

#include "../data/Data.h"
#include <boost/optional.hpp>

namespace cpputils {
    //TODO Test
    class RandomPadding final {
    public:
        static Data add(const Data &data, size_t targetSize);
        static boost::optional<Data> remove(const Data &data);
    };
}

#endif
