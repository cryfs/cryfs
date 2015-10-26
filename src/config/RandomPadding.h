#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_PADDING_H
#define MESSMER_CRYFS_SRC_CONFIG_PADDING_H

#include <messmer/cpp-utils/data/Data.h>
#include <boost/optional.hpp>

namespace cryfs {
    //TODO Test
    class RandomPadding {
    public:
        static cpputils::Data add(const cpputils::Data &data, size_t targetSize);
        static boost::optional<cpputils::Data> remove(const cpputils::Data &data);
    };
}

#endif
