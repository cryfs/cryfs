#include "testutils/FuseUtimensTest.h"

using ::testing::StrEq;
using ::testing::Return;
using ::testing::WithParamInterface;
using ::testing::Values;

class FuseUtimensTimeParameterTest: public FuseUtimensTest, public WithParamInterface<std::array<timespec, 2>> {
};
const std::array<timespec, 2> TIMEVAL1 = {FuseUtimensTest::makeTimespec(0,0), FuseUtimensTest::makeTimespec(0,0)};
const std::array<timespec, 2> TIMEVAL2 = {FuseUtimensTest::makeTimespec(1000,0), FuseUtimensTest::makeTimespec(0,0)};
const std::array<timespec, 2> TIMEVAL3 = {FuseUtimensTest::makeTimespec(0,1000), FuseUtimensTest::makeTimespec(0,0)};
const std::array<timespec, 2> TIMEVAL4 = {FuseUtimensTest::makeTimespec(1000,1000), FuseUtimensTest::makeTimespec(0,0)};
const std::array<timespec, 2> TIMEVAL5 = {FuseUtimensTest::makeTimespec(0,0), FuseUtimensTest::makeTimespec(0,0)};
const std::array<timespec, 2> TIMEVAL6 = {FuseUtimensTest::makeTimespec(0,0), FuseUtimensTest::makeTimespec(1000,0)};
const std::array<timespec, 2> TIMEVAL7 = {FuseUtimensTest::makeTimespec(0,0), FuseUtimensTest::makeTimespec(0,1000)};
const std::array<timespec, 2> TIMEVAL8 = {FuseUtimensTest::makeTimespec(0,0), FuseUtimensTest::makeTimespec(1000,1000)};
const std::array<timespec, 2> TIMEVAL9 = {FuseUtimensTest::makeTimespec(1417196126,123000), FuseUtimensTest::makeTimespec(1417109713,321000)}; // current timestamp and the day before as of writing this test case
const std::array<timespec, 2> TIMEVAL10 = {FuseUtimensTest::makeTimespec(UINT64_C(1024)*1024*1024*1024,999000), FuseUtimensTest::makeTimespec(UINT64_C(2*1024)*1024*1024*1024,321000)}; // needs 64bit for timestamp representation
INSTANTIATE_TEST_CASE_P(FuseUtimensTimeParameterTest, FuseUtimensTimeParameterTest,
    Values(TIMEVAL1, TIMEVAL2, TIMEVAL3, TIMEVAL4, TIMEVAL5, TIMEVAL6, TIMEVAL7, TIMEVAL8, TIMEVAL9, TIMEVAL10));


TEST_P(FuseUtimensTimeParameterTest, Utimens) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(*fsimpl, utimens(StrEq(FILENAME), TimeSpecEq(GetParam()[0]), TimeSpecEq(GetParam()[1])))
    .Times(1).WillOnce(Return());

  Utimens(FILENAME, GetParam()[0], GetParam()[1]);
}
