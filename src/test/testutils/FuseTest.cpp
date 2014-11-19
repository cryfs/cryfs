#include "FuseTest.h"

using ::testing::Action;
using ::testing::Invoke;

Action<void(const char*, struct ::stat*)> FuseTest::ReturnIsFileStat =
  Invoke([](const char*, struct ::stat* result) {
    result->st_mode = S_IFREG;
  });
