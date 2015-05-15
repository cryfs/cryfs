#pragma once
#ifndef BLOCKS_MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_CACHING_CACHE_TESTUTILS_MINIMALVALUETYPE_H_
#define BLOCKS_MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_CACHING_CACHE_TESTUTILS_MINIMALVALUETYPE_H_

#include <messmer/cpp-utils/macros.h>
#include <cassert>

// This is a not-default-constructible non-copyable but moveable Value type
class MinimalValueType {
public:
  static int instances;

  static MinimalValueType create(int value) {
    return MinimalValueType(value);
  }

  MinimalValueType(MinimalValueType &&rhs): MinimalValueType(rhs.value()) {
    rhs._isMoved = true;
  }

  ~MinimalValueType() {
    assert(!_isDestructed);
    --instances;
    _isDestructed = true;
  }

  int value() const {
    assert(!_isMoved && !_isDestructed);
    return _value;
  }

private:
  MinimalValueType(int value): _value(value), _isMoved(false), _isDestructed(false) {
    ++instances;
  }

  int _value;
  bool _isMoved;
  bool _isDestructed;

  DISALLOW_COPY_AND_ASSIGN(MinimalValueType);
};

#endif
