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
#include "FsppDeviceTest_Timestamps.h"
#include "FsppNodeTest_Timestamps.h"
#include "FsppDirTest_Timestamps.h"
#include "FsppSymlinkTest_Timestamps.h"
#include "FsppFileTest_Timestamps.h"
#include "FsppOpenFileTest_Timestamps.h"

#define FSPP_ADD_FILESYTEM_TESTS(FS_NAME, FIXTURE) \
  INSTANTIATE_TYPED_TEST_SUITE_P(FS_NAME, FsppDeviceTest_One,             FIXTURE);  \
  INSTANTIATE_TYPED_TEST_SUITE_P(FS_NAME, FsppDeviceTest_Two,             FIXTURE);  \
  INSTANTIATE_NODE_TEST_SUITE(   FS_NAME, FsppDeviceTest_Timestamps,      FIXTURE);  \
  INSTANTIATE_TYPED_TEST_SUITE_P(FS_NAME, FsppDirTest,                    FIXTURE);  \
  INSTANTIATE_TYPED_TEST_SUITE_P(FS_NAME, FsppDirTest_Timestamps,         FIXTURE);  \
  INSTANTIATE_NODE_TEST_SUITE(   FS_NAME, FsppDirTest_Timestamps_Entries, FIXTURE);  \
  INSTANTIATE_TYPED_TEST_SUITE_P(FS_NAME, FsppFileTest,                   FIXTURE);  \
  INSTANTIATE_TYPED_TEST_SUITE_P(FS_NAME, FsppFileTest_Timestamps,        FIXTURE);  \
  INSTANTIATE_TYPED_TEST_SUITE_P(FS_NAME, FsppSymlinkTest,                FIXTURE);  \
  INSTANTIATE_TYPED_TEST_SUITE_P(FS_NAME, FsppSymlinkTest_Timestamps,     FIXTURE);  \
  INSTANTIATE_NODE_TEST_SUITE(   FS_NAME, FsppNodeTest_Rename,            FIXTURE);  \
  INSTANTIATE_NODE_TEST_SUITE(   FS_NAME, FsppNodeTest_Stat,              FIXTURE);  \
  INSTANTIATE_NODE_TEST_SUITE(   FS_NAME, FsppNodeTest_Timestamps,        FIXTURE);  \
  INSTANTIATE_TYPED_TEST_SUITE_P(FS_NAME, FsppNodeTest_Stat_FileOnly,     FIXTURE);  \
  INSTANTIATE_TYPED_TEST_SUITE_P(FS_NAME, FsppNodeTest_Stat_DirOnly,      FIXTURE);  \
  INSTANTIATE_TYPED_TEST_SUITE_P(FS_NAME, FsppNodeTest_Stat_SymlinkOnly,  FIXTURE);  \
  INSTANTIATE_TYPED_TEST_SUITE_P(FS_NAME, FsppOpenFileTest,               FIXTURE);  \
  INSTANTIATE_TYPED_TEST_SUITE_P(FS_NAME, FsppOpenFileTest_Timestamps,    FIXTURE);  \


#endif
