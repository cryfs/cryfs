#pragma once
#ifndef MESSMER_FSPP_TEST_FUSE_STATFS_TESTUTILS_FUSESTATFSRETURNTEST_H_
#define MESSMER_FSPP_TEST_FUSE_STATFS_TESTUTILS_FUSESTATFSRETURNTEST_H_

#include "FuseStatfsTest.h"

// This class offers test helpers for testing (struct statfs) entries. We return them from
// our mock filesystem, set up a temporary filesystem, call statfs syscall on it, and
// then check the return value.
template<typename Property>
class FuseStatfsReturnTest: public FuseStatfsTest {
public:
  // Set the specified (struct statfs) entry to the given value, and test whether it is correctly returned from the syscall.
  struct ::statvfs CallStatfsWithValue(Property value);

private:
  std::function<void(struct ::statvfs*)> SetPropertyImpl(Property value);

  // Override this function to specify, how to set the specified (struct statfs) entry on the passed (struct statfs *) object.
  virtual void set(struct ::statvfs *statfs, Property value) = 0;
};

template<typename Property>
inline struct ::statvfs FuseStatfsReturnTest<Property>::CallStatfsWithValue(Property value) {
  return CallStatfsWithImpl(SetPropertyImpl(value));
}

template<typename Property>
inline std::function<void(struct ::statvfs*)> FuseStatfsReturnTest<Property>::SetPropertyImpl(Property value) {
  return [this, value] (struct ::statvfs *stat) {
    set(stat, value);
  };
}


#endif
