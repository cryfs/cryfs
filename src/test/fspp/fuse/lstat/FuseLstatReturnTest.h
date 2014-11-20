#pragma once
#ifndef TEST_FSPP_FUSE_LSTAT_FUSELSTATRETURNTEST_H_
#define TEST_FSPP_FUSE_LSTAT_FUSELSTATRETURNTEST_H_

#include "FuseLstatTest.h"

template<typename Property>
class FuseLstatReturnPropertyTest: public FuseLstatTest {
public:
  struct stat CallFileLstatWithValue(Property value) {
    return CallFileLstatWithImpl(SetPropertyImpl(value));
  }
  struct stat CallDirLstatWithValue(Property value) {
    return CallDirLstatWithImpl(SetPropertyImpl(value));
  }
private:
  std::function<void(struct stat*)> SetPropertyImpl(Property value) {
    return [this, value] (struct stat *stat) {
      set(stat, value);
    };
  }
  virtual void set(struct stat *stat, Property value) = 0;
};


#endif
