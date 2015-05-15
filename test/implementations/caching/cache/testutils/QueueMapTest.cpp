#include "QueueMapTest.h"

QueueMapTest::QueueMapTest() {
  MinimalKeyType::instances = 0;
  MinimalValueType::instances = 0;
  _map = std::make_unique<blockstore::caching::QueueMap<MinimalKeyType, MinimalValueType>>();
}

QueueMapTest::~QueueMapTest() {
  _map.reset();
  EXPECT_EQ(0, MinimalKeyType::instances);
  EXPECT_EQ(0, MinimalValueType::instances);
}

void QueueMapTest::push(int key, int value) {
  _map->push(MinimalKeyType::create(key), MinimalValueType::create(value));
}

boost::optional<int> QueueMapTest::pop() {
  auto elem = _map->pop();
  if (!elem) {
    return boost::none;
  }
  return elem.value().value();
}

boost::optional<int> QueueMapTest::pop(int key) {
  auto elem = _map->pop(MinimalKeyType::create(key));
  if (!elem) {
    return boost::none;
  }
  return elem.value().value();
}

boost::optional<int> QueueMapTest::peek() {
  auto elem = _map->peek();
  if (!elem) {
    return boost::none;
  }
  return elem.value().value();
}

int QueueMapTest::size() {
  return _map->size();
}
