#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPOPENFILETEST_H_
#define MESSMER_FSPP_FSTEST_FSPPOPENFILETEST_H_

#include "testutils/FileTest.h"

template<class ConcreteFileSystemTestFixture>
class FsppOpenFileTest: public FileTest<ConcreteFileSystemTestFixture> {
public:
};

TYPED_TEST_CASE_P(FsppOpenFileTest);

TYPED_TEST_P(FsppOpenFileTest, Bla) {
  //TODO
}

REGISTER_TYPED_TEST_CASE_P(FsppOpenFileTest,
  Bla
);

//TODO Test stat
//TODO Test truncate
//TODO Test read
//TODO Test write
//TODO Test flush
//TODO Test fsync
//TODO Test fdatasync
//TODO Test stat on file that was just created (i.e. the OpenFile instance returned by createAndOpenFile)
//TODO Test all operations do (or don't) affect file timestamps correctly

#endif
