#pragma once
#ifndef TEST_FSPP_FUSE_LSTAT_FUSELSTATRETURNTEST_H_
#define TEST_FSPP_FUSE_LSTAT_FUSELSTATRETURNTEST_H_

#include "FuseLstatTest.h"

template<typename Property>
class FuseLstatReturnTest: public FuseLstatTest {
public:
  struct stat CallFileLstatWithValue(Property value);
  struct stat CallDirLstatWithValue(Property value);

private:
  std::function<void(struct stat*)> SetPropertyImpl(Property value);

  virtual void set(struct stat *stat, Property value) = 0;
};

template<typename Property>
struct stat FuseLstatReturnTest<Property>::CallFileLstatWithValue(Property value) {
  return CallFileLstatWithImpl(SetPropertyImpl(value));
}

template<typename Property>
struct stat FuseLstatReturnTest<Property>::CallDirLstatWithValue(Property value) {
  return CallDirLstatWithImpl(SetPropertyImpl(value));
}

template<typename Property>
std::function<void(struct stat*)> FuseLstatReturnTest<Property>::SetPropertyImpl(Property value) {
  return [this, value] (struct stat *stat) {
    set(stat, value);
  };
}


#endif
