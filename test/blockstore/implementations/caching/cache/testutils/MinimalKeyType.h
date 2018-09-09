#pragma once
#ifndef MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_CACHING_CACHE_TESTUTILS_MINIMALKEYTYPE_H_
#define MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_CACHING_CACHE_TESTUTILS_MINIMALKEYTYPE_H_

#include <unordered_map>
#include <atomic>

// This is a not-default-constructible Key type
class MinimalKeyType {
public:
  static std::atomic<int> instances;

  static MinimalKeyType create(int value) {
    return MinimalKeyType(value);
  }

  MinimalKeyType(const MinimalKeyType &rhs): MinimalKeyType(rhs.value()) {
  }

  ~MinimalKeyType() {
    --instances;
  }

  int value() const {
    return _value;
  }

private:
  MinimalKeyType(int value): _value(value) {
    ++instances;
  }

  int _value;
};

namespace std {
template <> struct hash<MinimalKeyType> {
  size_t operator()(const MinimalKeyType &obj) const {
    return obj.value();
  }
};
}

inline bool operator==(const MinimalKeyType &lhs, const MinimalKeyType &rhs) {
  return lhs.value() == rhs.value();
}

#endif
