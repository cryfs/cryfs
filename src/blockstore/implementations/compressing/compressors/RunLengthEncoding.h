#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_COMPRESSING_COMPRESSORS_RUNLENGTHENCODING_H
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_COMPRESSING_COMPRESSORS_RUNLENGTHENCODING_H

#include <cpp-utils/data/Data.h>

namespace blockstore {
    namespace compressing {
        class RunLengthEncoding {
        public:
            static cpputils::Data Compress(const cpputils::Data &data);

            static cpputils::Data Decompress(const void *data, size_t size);

        private:
            static void _encodeArbitraryWords(const uint8_t **current, const uint8_t* end, std::ostringstream *output);
            static uint16_t _arbitraryRunLength(const uint8_t *start, const uint8_t* end);
            static void _encodeIdenticalWords(const uint8_t **current, const uint8_t* end, std::ostringstream *output);
            static uint16_t _countIdenticalBytes(const uint8_t *start, const uint8_t *end);
            static bool _hasData(std::istringstream *stream);
            static cpputils::Data _extractData(std::ostringstream *stream);
            static void _parseData(const uint8_t *data, size_t size, std::istringstream *result);
            static void _decodeArbitraryWords(std::istringstream *stream, std::ostringstream *decompressed);
            static void _decodeIdenticalWords(std::istringstream *stream, std::ostringstream *decompressed);
        };
    }
}

#endif
