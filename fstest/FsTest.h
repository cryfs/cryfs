#ifndef MESSMER_FSPP_FSTEST_FSTEST_H_
#define MESSMER_FSPP_FSTEST_FSTEST_H_

#include "testutils/FileSystemTest.h"
#include "FsppDeviceTest.h"
#include "FsppDirTest.h"

#define FSPP_ADD_FILESYTEM_TESTS(FS_NAME, FIXTURE) \
  INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, FsppDeviceTest, FIXTURE);    \
  INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, FsppDirTest,    FIXTURE);

//TODO ...

#endif
