#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSTEST_H_
#define MESSMER_FSPP_FSTEST_FSTEST_H_

#include "testutils/FileSystemTest.h"
#include "FsppDeviceTest.h"
#include "FsppDirTest.h"
#include "FsppFileTest.h"
#include "FsppOpenFileTest.h"

#define FSPP_ADD_FILESYTEM_TESTS(FS_NAME, FIXTURE) \
  INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, FsppDeviceTest,   FIXTURE);  \
  INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, FsppDirTest,      FIXTURE);  \
  INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, FsppFileTest,     FIXTURE);  \
  INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, FsppOpenFileTest, FIXTURE);  \

#endif
