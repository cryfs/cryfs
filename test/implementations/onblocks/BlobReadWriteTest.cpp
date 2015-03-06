#include "testutils/BlobStoreTest.h"
#include <messmer/blockstore/utils/Data.h>

//TODO Test read/write
//TODO Test read/write with loading inbetween
//TODO Test flushing isn't necessary (writing immediately flushes)

using std::unique_ptr;

using namespace blobstore;
using blockstore::Key;
using blockstore::Data;

class BlobReadWriteTest: public BlobStoreTest {
public:
};
