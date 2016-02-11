#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_COMPRESSING_COMPRESSORS_GZIP_H
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_COMPRESSING_COMPRESSORS_GZIP_H

#include <cpp-utils/data/Data.h>

namespace blockstore {
    namespace compressing {
        class Gzip {
        public:
            static cpputils::Data Compress(const cpputils::Data &data);

            static cpputils::Data Decompress(const void *data, size_t size);
        };
    }
}

#endif
