#include "CacheTest.h"

void CacheTest::push(int key, int value) {
  return _cache.push(MinimalKeyType::create(key), MinimalValueType::create(value));
}

boost::optional<int> CacheTest::pop(int key) {
  boost::optional<MinimalValueType> entry = _cache.pop(MinimalKeyType::create(key));
  if (!entry) {
    return boost::none;
  }
  return entry->value();
}
