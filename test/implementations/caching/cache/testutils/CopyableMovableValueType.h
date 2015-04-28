#pragma once
#ifndef MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_CACHING_CACHE_TESTUTILS_COPYABLEMOVABLEVALUETYPE_H_
#define MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_CACHING_CACHE_TESTUTILS_COPYABLEMOVABLEVALUETYPE_H_

class CopyableMovableValueType {
public:
  static int numCopyConstructorCalled;
  CopyableMovableValueType(int value): _value(value) {}
  CopyableMovableValueType(const CopyableMovableValueType &rhs): CopyableMovableValueType(rhs._value) {
    ++numCopyConstructorCalled;
  }
  CopyableMovableValueType(CopyableMovableValueType &&rhs): CopyableMovableValueType(rhs._value) {
    //Don't increase numCopyConstructorCalled
  }
  int value() const {
    return _value;
  }
private:
  int _value;
};


#endif
