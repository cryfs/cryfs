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
  // NOLINTNEXTLINE(cert-oop54-cpp)
  CopyableMovableValueType &operator=(const CopyableMovableValueType &rhs) {
    _value = rhs._value;
    ++numCopyConstructorCalled;
    return *this;
  }
  CopyableMovableValueType(CopyableMovableValueType &&rhs) noexcept: CopyableMovableValueType(rhs._value) {
    //Don't increase numCopyConstructorCalled
  }
  CopyableMovableValueType &operator=(CopyableMovableValueType &&rhs) noexcept {
    //Don't increase numCopyConstructorCalled
    _value = rhs._value;
    return *this;
  }
  int value() const {
    return _value;
  }
private:
  int _value;
};


#endif
