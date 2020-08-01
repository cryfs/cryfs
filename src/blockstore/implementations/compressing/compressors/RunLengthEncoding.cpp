#include "RunLengthEncoding.h"
#include <sstream>
#include <cpp-utils/assert/assert.h>

using cpputils::Data;
using std::string;
using std::ostringstream;
using std::istringstream;

namespace blockstore {
    namespace compressing {

        // Alternatively store a run of arbitrary bytes and a run of identical bytes.
        // Each run is preceded by its length. Length fields are uint16_t.
        // Example: 2 - 5 - 8 - 10 - 3 - 0 - 2 - 0
        // Length 2 arbitrary bytes (values: 5, 8), the next 10 bytes store "3" each,
        // then 0 arbitrary bytes and 2x "0".

        Data RunLengthEncoding::Compress(const Data &data) {
            ostringstream compressed;
            const uint8_t *current = static_cast<const uint8_t*>(data.data());
            const uint8_t *end = static_cast<const uint8_t*>(data.data())+data.size();
            while (current < end) {
                _encodeArbitraryWords(&current, end, &compressed);
                ASSERT(current <= end, "Overflow");
                if (current == end) {
                    break;
                }
                _encodeIdenticalWords(&current, end, &compressed);
                ASSERT(current <= end, "Overflow");
            }
            return _extractData(&compressed);
        }

        void RunLengthEncoding::_encodeArbitraryWords(const uint8_t **current, const uint8_t* end, ostringstream *output) {
            uint16_t size = _arbitraryRunLength(*current, end);
            output->write(reinterpret_cast<const char*>(&size), sizeof(uint16_t));
            output->write(reinterpret_cast<const char*>(*current), size);
            *current += size;
        }

        uint16_t RunLengthEncoding::_arbitraryRunLength(const uint8_t *start, const uint8_t* end) {
            // Each stopping of an arbitrary bytes run costs us 5 byte, because we have to store the length
            // for the identical bytes run (2 byte), the identical byte itself (1 byte) and the length for the next arbitrary bytes run (2 byte).
            // So to get an advantage from stopping an arbitrary bytes run, at least 6 bytes have to be identical.

            // realEnd avoids an overflow of the 16bit counter
            const uint8_t *realEnd = std::min(end, start + std::numeric_limits<uint16_t>::max());

            // Count the number of identical bytes and return if it finds a run of more than 6 identical bytes.
            uint8_t lastByte = *start + 1; // Something different from the first byte
            uint8_t numIdenticalBytes = 1;
            for(const uint8_t *current = start; current != realEnd; ++current) {
                if (*current == lastByte) {
                    ++numIdenticalBytes;
                    if (numIdenticalBytes == 6) {
                        return current - start - 5; //-5, because the end pointer for the arbitrary byte run should point to the first identical byte, not the one before.
                    }
                } else {
                    numIdenticalBytes = 1;
                }
                lastByte = *current;
            }
            //It wasn't worth stopping the arbitrary bytes run anywhere. The whole region should be an arbitrary run.
            return realEnd-start;
        }

        void RunLengthEncoding::_encodeIdenticalWords(const uint8_t **current, const uint8_t* end, ostringstream *output) {
            uint16_t size = _countIdenticalBytes(*current, end);
            output->write(reinterpret_cast<const char*>(&size), sizeof(uint16_t));
            output->write(reinterpret_cast<const char*>(*current), 1);
            *current += size;
        }

        uint16_t RunLengthEncoding::_countIdenticalBytes(const uint8_t *start, const uint8_t *end) {
            const uint8_t *realEnd = std::min(end, start + std::numeric_limits<uint16_t>::max()); // This prevents overflow of the 16bit counter
            for (const uint8_t *current = start+1; current != realEnd; ++current) {
                if (*current != *start) {
                    return current-start;
                }
            }
            // All bytes have been identical
            return realEnd - start;
        }

        Data RunLengthEncoding::_extractData(ostringstream *stream) {
            string str = stream->str();
            Data data(str.size());
            std::memcpy(data.data(), str.c_str(), str.size());
            return data;
        }

        Data RunLengthEncoding::Decompress(const void *data, size_t size) {
            istringstream stream;
            _parseData(static_cast<const uint8_t*>(data), size, &stream);
            ostringstream decompressed;
            while(_hasData(&stream)) {
                _decodeArbitraryWords(&stream, &decompressed);
                if (!_hasData(&stream)) {
                    break;
                }
                _decodeIdenticalWords(&stream, &decompressed);
            }
            return _extractData(&decompressed);
        }

        bool RunLengthEncoding::_hasData(istringstream *str) {
            str->peek();
            return !str->eof();
        }

        void RunLengthEncoding::_parseData(const uint8_t *data, size_t size, istringstream *result) {
            result->str(string(reinterpret_cast<const char*>(data), size));
        }

        void RunLengthEncoding::_decodeArbitraryWords(istringstream *stream, ostringstream *decompressed) {
            uint16_t size = 0;
            stream->read(reinterpret_cast<char*>(&size), sizeof(uint16_t));
            ASSERT(stream->good(), "Premature end of stream");
            Data run(size);
            stream->read(static_cast<char*>(run.data()), size);
            ASSERT(stream->good(), "Premature end of stream");
            decompressed->write(static_cast<const char*>(run.data()), run.size());
        }

        void RunLengthEncoding::_decodeIdenticalWords(istringstream *stream, ostringstream *decompressed) {
            uint16_t size = 0;
            stream->read(reinterpret_cast<char*>(&size), sizeof(uint16_t));
            ASSERT(stream->good(), "Premature end of stream");
            uint8_t value = 0;
            stream->read(reinterpret_cast<char*>(&value), 1);
            ASSERT(stream->good(), "Premature end of stream");
            Data run(size);
            std::memset(run.data(), value, run.size());
            decompressed->write(static_cast<const char*>(run.data()), run.size());
        }

    }
}
