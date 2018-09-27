#pragma once
#ifndef MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_CACHING_CACHE_TESTUTILS_MINIMALVALUETYPE_H_
#define MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_CACHING_CACHE_TESTUTILS_MINIMALVALUETYPE_H_

#include <cpp-utils/macros.h>
#include <cassert>
#include <cpp-utils/assert/assert.h>
#include <atomic>

// This is a not-default-constructible non-copyable but moveable Value type
class MinimalValueType {
public:
  static std::atomic<int> instances;

  static MinimalValueType create(int value) {
    return MinimalValueType(value);
  }

  MinimalValueType(MinimalValueType &&rhs) noexcept: MinimalValueType(rhs.value()) {
    rhs._isMoved = true;
  }

  MinimalValueType &operator=(MinimalValueType &&rhs) noexcept {
    _value = rhs.value();
    _isMoved = false;
    rhs._isMoved = true;
    return *this;
  }

  ~MinimalValueType() {
    ASSERT(!_isDestructed, "Object was already destructed before");
    --instances;
    _isDestructed = true;
  }

  int value() const {
    ASSERT(!_isMoved && !_isDestructed, "Object is invalid");
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
