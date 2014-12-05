#include "gtest/gtest.h"

#include "test/testutils/TempFile.h"

#include "blobstore/implementations/ondisk/OnDiskBlob.h"

using ::testing::Test;

using std::unique_ptr;

using blobstore::ondisk::OnDiskBlob;

namespace bf = boost::filesystem;

class OnDiskBlobLoadTest: public Test {
public:
  TempFile file;
};

