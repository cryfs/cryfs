#pragma once
#ifndef MESSMER_FSPP_TEST_FUSE_STATFS_TESTUTILS_FUSESTATFSTEST_H_
#define MESSMER_FSPP_TEST_FUSE_STATFS_TESTUTILS_FUSESTATFSTEST_H_

#include <string>
#include <functional>

#include "../../../testutils/FuseTest.h"

// This class offers some utility functions for testing statfs().
class FuseStatfsTest: public FuseTest {
protected:
  const char *FILENAME = "/myfile";

  // Set up a temporary filesystem (using the fsimpl mock in FuseTest as filesystem implementation)
  // and call the statfs syscall on the given (filesystem-relative) path.
  void Statfs(const std::string &path);
  // Same as Statfs above, but also return the result of the statfs syscall.
  void Statfs(const std::string &path, struct ::statvfs *result);

  // These two functions are the same as Statfs above, but they don't fail the test when the statfs syscall
  // crashes. Instead, they return the result value of the statfs syscall.
  int StatfsReturnError(const std::string &path);
  int StatfsReturnError(const std::string &path, struct ::statvfs *result);

  // You can specify an implementation, which can modify the (struct statfs *) result,
  // our fuse mock filesystem implementation will then return this to fuse on an statfs call.
  // This functions then set up a temporary filesystem with this mock, calls statfs on a filesystem node
  // and returns the (struct statfs) returned from an statfs syscall to this filesystem.
  struct ::statvfs CallStatfsWithImpl(std::function<void(struct ::statvfs*)> implementation);
};

#endif
