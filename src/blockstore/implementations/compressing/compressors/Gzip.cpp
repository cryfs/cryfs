#include "Gzip.h"
#include <cryptopp/gzip.h>

using cpputils::Data;

namespace blockstore {
    namespace compressing {

        Data Gzip::Compress(const Data &data) {
            CryptoPP::Gzip zipper;
            zipper.Put((byte *) data.data(), data.size());
            zipper.MessageEnd();
            Data compressed(zipper.MaxRetrievable());
            zipper.Get((byte *) compressed.data(), compressed.size());
            return compressed;
        }

        Data Gzip::Decompress(const void *data, size_t size) {
            //TODO Change interface to taking cpputils::Data objects (needs changing blockstore so we can read their "class Data", because this is called from CompressedBlock::Decompress()).
            CryptoPP::Gunzip zipper;
            zipper.Put((byte *) data, size);
            zipper.MessageEnd();
            Data decompressed(zipper.MaxRetrievable());
            zipper.Get((byte *) decompressed.data(), decompressed.size());
            return decompressed;
        }

    }
}