#include "cpp-utils/crypto/cryptopp_byte.h"
#include "Gzip.h"
#include <cryptopp/gzip.h>

using cpputils::Data;

namespace blockstore {
    namespace compressing {

        Data Gzip::Compress(const Data &data) {
            CryptoPP::Gzip zipper;
            zipper.Put((CryptoPP::byte *) data.data(), data.size());
            zipper.MessageEnd();
            Data compressed(zipper.MaxRetrievable());
            zipper.Get((CryptoPP::byte *) compressed.data(), compressed.size());
            return compressed;
        }

        Data Gzip::Decompress(const void *data, size_t size) {
            //TODO Change interface to taking cpputils::Data objects (needs changing blockstore so we can read their "class Data", because this is called from CompressedBlock::Decompress()).
            CryptoPP::Gunzip zipper;
            zipper.Put((CryptoPP::byte *) data, size);
            zipper.MessageEnd();
            Data decompressed(zipper.MaxRetrievable());
            zipper.Get((CryptoPP::byte *) decompressed.data(), decompressed.size());
            return decompressed;
        }

    }
}
