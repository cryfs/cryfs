#include "testutils/FuseUtimensTest.h"

using ::testing::StrEq;
using ::testing::Return;
using ::testing::WithParamInterface;
using ::testing::Values;

class FuseUtimensTimeParameterTest: public FuseUtimensTest, public WithParamInterface<const timespec*> {
};
const timespec TIMEVAL1[2] = {FuseUtimensTest::makeTimespec(0,0), FuseUtimensTest::makeTimespec(0,0)};
const timespec TIMEVAL2[2] = {FuseUtimensTest::makeTimespec(1000,0), FuseUtimensTest::makeTimespec(0,0)};
const timespec TIMEVAL3[2] = {FuseUtimensTest::makeTimespec(0,1000), FuseUtimensTest::makeTimespec(0,0)};
const timespec TIMEVAL4[2] = {FuseUtimensTest::makeTimespec(1000,1000), FuseUtimensTest::makeTimespec(0,0)};
const timespec TIMEVAL5[2] = {FuseUtimensTest::makeTimespec(0,0), FuseUtimensTest::makeTimespec(0,0)};
const timespec TIMEVAL6[2] = {FuseUtimensTest::makeTimespec(0,0), FuseUtimensTest::makeTimespec(1000,0)};
const timespec TIMEVAL7[2] = {FuseUtimensTest::makeTimespec(0,0), FuseUtimensTest::makeTimespec(0,1000)};
const timespec TIMEVAL8[2] = {FuseUtimensTest::makeTimespec(0,0), FuseUtimensTest::makeTimespec(1000,1000)};
const timespec TIMEVAL9[2] = {FuseUtimensTest::makeTimespec(1417196126,123000), FuseUtimensTest::makeTimespec(1417109713,321000)}; // current timestamp and the day before as of writing this test case
const timespec TIMEVAL10[2] = {FuseUtimensTest::makeTimespec(UINT64_C(1024)*1024*1024*1024,999000), FuseUtimensTest::makeTimespec(UINT64_C(2*1024)*1024*1024*1024,321000)}; // needs 64bit for timestamp representation
INSTANTIATE_TEST_CASE_P(FuseUtimensTimeParameterTest, FuseUtimensTimeParameterTest,
    Values(TIMEVAL1, TIMEVAL2, TIMEVAL3, TIMEVAL4, TIMEVAL5, TIMEVAL6, TIMEVAL7, TIMEVAL8, TIMEVAL9, TIMEVAL10));


TEST_P(FuseUtimensTimeParameterTest, Utimens) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(fsimpl, utimens(StrEq(FILENAME), TimeSpecEq(GetParam()[0]), TimeSpecEq(GetParam()[1])))
    .Times(1).WillOnce(Return());

  Utimens(FILENAME, GetParam()[0], GetParam()[1]);
}
