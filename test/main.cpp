#include "google/gtest/gtest.h"
#include <messmer/cpp-utils/assert/backtrace.h>

int main(int argc, char **argv) {
  cpputils::showBacktraceOnSigSegv();
  testing::InitGoogleTest(&argc, argv);
  return RUN_ALL_TESTS();
}
