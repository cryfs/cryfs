#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSTEST_H_
#define MESSMER_FSPP_FSTEST_FSTEST_H_

#include "testutils/FileSystemTest.h"
#include "FsppDeviceTest.h"
#include "FsppDirTest.h"
#include "FsppFileTest.h"
#include "FsppSymlinkTest.h"
#include "FsppNodeTest_Rename.h"
#include "FsppNodeTest_Stat.h"
#include "FsppOpenFileTest.h"

#define FSPP_ADD_FILESYTEM_TESTS(FS_NAME, FIXTURE) \
  INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, FsppDeviceTest,                FIXTURE);  \
  INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, FsppDirTest,                   FIXTURE);  \
  INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, FsppFileTest,                  FIXTURE);  \
  INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, FsppSymlinkTest,               FIXTURE);  \
  INSTANTIATE_NODE_TEST_CASE(   FS_NAME, FsppNodeTest_Rename,           FIXTURE);  \
  INSTANTIATE_NODE_TEST_CASE(   FS_NAME, FsppNodeTest_Stat,             FIXTURE);  \
  INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, FsppNodeTest_Stat_FileOnly,    FIXTURE);  \
  INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, FsppNodeTest_Stat_DirOnly,     FIXTURE);  \
  INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, FsppNodeTest_Stat_SymlinkOnly, FIXTURE);  \
  INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, FsppOpenFileTest,              FIXTURE);  \

#endif
