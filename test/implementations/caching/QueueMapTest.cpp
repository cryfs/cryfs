#include <google/gtest/gtest.h>
#include <messmer/cpp-utils/macros.h>
#include "../../../implementations/caching/QueueMap.h"
#include <memory>
#include <boost/optional/optional_io.hpp>

using ::testing::Test;
using std::unique_ptr;
using std::make_unique;

using namespace blockstore::caching;

// This is a not-default-constructible Key type
class MinimalKeyType {
public:
  static int instances;
  static MinimalKeyType create(int value) {
    return MinimalKeyType(value);
  }
  bool operator==(const MinimalKeyType &rhs) const {
    return _value == rhs._value;
  }
  int value() const {
    return _value;
  }
  MinimalKeyType(const MinimalKeyType &rhs): MinimalKeyType(rhs.value()) {
  }
  ~MinimalKeyType() {
    --instances;
  }
private:
  MinimalKeyType(int value): _value(value) {
    ++instances;
  }
  int _value;
};
int MinimalKeyType::instances = 0;
namespace std {
template <> struct hash<MinimalKeyType> {
  size_t operator()(const MinimalKeyType &obj) const {
    return obj.value();
  }
};
}
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
int MinimalValueType::instances = 0;

class QueueMapTest: public Test {
public:
  QueueMapTest() {
    MinimalKeyType::instances = 0;
    MinimalValueType::instances = 0;
    _map = make_unique<QueueMap<MinimalKeyType, MinimalValueType>>();
  }
  ~QueueMapTest() {
    _map.reset();
    EXPECT_EQ(0, MinimalKeyType::instances);
    EXPECT_EQ(0, MinimalValueType::instances);
  }
  void push(int key, int value) {
    _map->push(MinimalKeyType::create(key), MinimalValueType::create(value));
  }
  boost::optional<int> pop() {
    auto elem = _map->pop();
    if (!elem) {
      return boost::none;
    }
    return elem.value().value();
  }
  boost::optional<int> pop(int key) {
    auto elem = _map->pop(MinimalKeyType::create(key));
    if (!elem) {
      return boost::none;
    }
    return elem.value().value();
  }
  boost::optional<int> peek() {
    auto elem = _map->peek();
    if (!elem) {
      return boost::none;
    }
    return elem.value().value();
  }
  int size() {
    return _map->size();
  }
private:
  unique_ptr<QueueMap<MinimalKeyType, MinimalValueType>> _map;
};

class QueueMapSizeTest: public QueueMapTest {};

TEST_F(QueueMapSizeTest, Empty) {
  EXPECT_EQ(0, size());
}

TEST_F(QueueMapSizeTest, AfterPushingOne) {
  push(2, 3);
  EXPECT_EQ(1, size());
}

TEST_F(QueueMapSizeTest, AfterPushingTwo) {
  push(2, 3);
  push(3, 4);
  EXPECT_EQ(2, size());
}

TEST_F(QueueMapSizeTest, AfterPushingTwoAndPoppingOldest) {
  push(2, 3);
  push(3, 4);
  pop();
  EXPECT_EQ(1, size());
}

TEST_F(QueueMapSizeTest, AfterPushingTwoAndPoppingFirst) {
  push(2, 3);
  push(3, 4);
  pop(2);
  EXPECT_EQ(1, size());
}

TEST_F(QueueMapSizeTest, AfterPushingTwoAndPoppingLast) {
  push(2, 3);
  push(3, 4);
  pop(3);
  EXPECT_EQ(1, size());
}

TEST_F(QueueMapSizeTest, AfterPushingOnePoppingOne) {
  push(2, 3);
  pop();
  EXPECT_EQ(0, size());
}

TEST_F(QueueMapSizeTest, AfterPushingOnePoppingOnePerKey) {
  push(2, 3);
  pop(2);
  EXPECT_EQ(0, size());
}

TEST_F(QueueMapSizeTest, AfterPushingOnePoppingOnePushingOne) {
  push(2, 3);
  pop();
  push(3, 4);
  EXPECT_EQ(1, size());
}

TEST_F(QueueMapSizeTest, AfterPushingOnePoppingOnePerKeyPushingOne) {
  push(2, 3);
  pop(2);
  push(3, 4);
  EXPECT_EQ(1, size());
}

TEST_F(QueueMapSizeTest, AfterPushingOnePoppingOnePushingSame) {
  push(2, 3);
  pop();
  push(2, 3);
  EXPECT_EQ(1, size());
}

TEST_F(QueueMapSizeTest, AfterPushingOnePoppingOnePerKeyPushingSame) {
  push(2, 3);
  pop(2);
  push(2, 3);
  EXPECT_EQ(1, size());
}

class QueueMapMemoryLeakTest: public QueueMapTest {
public:
  void EXPECT_NUM_INSTANCES(int num) {
    EXPECT_EQ(num, MinimalKeyType::instances);
    EXPECT_EQ(num, MinimalValueType::instances);
  }
};

TEST_F(QueueMapMemoryLeakTest, Empty) {
  EXPECT_NUM_INSTANCES(0);
}

TEST_F(QueueMapMemoryLeakTest, AfterPushingOne) {
  push(2, 3);
  EXPECT_NUM_INSTANCES(1);
}

TEST_F(QueueMapMemoryLeakTest, AfterPushingTwo) {
  push(2, 3);
  push(3, 4);
  EXPECT_NUM_INSTANCES(2);
}

TEST_F(QueueMapMemoryLeakTest, AfterPushingTwoAndPoppingOldest) {
  push(2, 3);
  push(3, 4);
  pop();
  EXPECT_NUM_INSTANCES(1);
}

TEST_F(QueueMapMemoryLeakTest, AfterPushingTwoAndPoppingFirst) {
  push(2, 3);
  push(3, 4);
  pop(2);
  EXPECT_NUM_INSTANCES(1);
}

TEST_F(QueueMapMemoryLeakTest, AfterPushingTwoAndPoppingLast) {
  push(2, 3);
  push(3, 4);
  pop(3);
  EXPECT_NUM_INSTANCES(1);
}

TEST_F(QueueMapMemoryLeakTest, AfterPushingOnePoppingOne) {
  push(2, 3);
  pop();
  EXPECT_NUM_INSTANCES(0);
}

TEST_F(QueueMapMemoryLeakTest, AfterPushingOnePoppingOnePerKey) {
  push(2, 3);
  pop(2);
  EXPECT_NUM_INSTANCES(0);
}

TEST_F(QueueMapMemoryLeakTest, AfterPushingOnePoppingOnePushingOne) {
  push(2, 3);
  pop();
  push(3, 4);
  EXPECT_NUM_INSTANCES(1);
}

TEST_F(QueueMapMemoryLeakTest, AfterPushingOnePoppingOnePerKeyPushingOne) {
  push(2, 3);
  pop(2);
  push(3, 4);
  EXPECT_NUM_INSTANCES(1);
}

TEST_F(QueueMapMemoryLeakTest, AfterPushingOnePoppingOnePushingSame) {
  push(2, 3);
  pop();
  push(2, 3);
  EXPECT_NUM_INSTANCES(1);
}

TEST_F(QueueMapMemoryLeakTest, AfterPushingOnePoppingOnePerKeyPushingSame) {
  push(2, 3);
  pop(2);
  push(2, 3);
  EXPECT_NUM_INSTANCES(1);
}

class QueueMapValueTest: public QueueMapTest {};

TEST_F(QueueMapValueTest, PoppingFromEmpty) {
  EXPECT_EQ(boost::none, pop());
}

TEST_F(QueueMapValueTest, PoppingFromEmptyPerKey) {
  EXPECT_EQ(boost::none, pop(2));
}

TEST_F(QueueMapValueTest, PoppingNonexistingPerKey) {
  push(3, 2);
  EXPECT_EQ(boost::none, pop(2));
}

TEST_F(QueueMapValueTest, PushingOne) {
  push(3, 2);
  EXPECT_EQ(2, pop(3).value());
  EXPECT_EQ(boost::none, pop());
}

TEST_F(QueueMapValueTest, PushingTwo) {
  push(2, 3);
  push(3, 4);
  EXPECT_EQ(3, pop().value());
  EXPECT_EQ(4, pop().value());
  EXPECT_EQ(boost::none, pop());
}

TEST_F(QueueMapValueTest, AfterPushingTwoAndPoppingFirst) {
  push(2, 3);
  push(3, 4);
  pop(2);
  EXPECT_EQ(boost::none, pop(2));
  EXPECT_EQ(4, pop(3).value());
  EXPECT_EQ(boost::none, pop());
}

TEST_F(QueueMapValueTest, AfterPushingTwoAndPoppingLast) {
  push(2, 3);
  push(3, 4);
  pop(3);
  EXPECT_EQ(boost::none, pop(3));
  EXPECT_EQ(3, pop(2).value());
  EXPECT_EQ(boost::none, pop());
}

TEST_F(QueueMapValueTest, AfterPushingOnePoppingOne) {
  push(2, 3);
  pop();
  EXPECT_EQ(boost::none, pop());
  EXPECT_EQ(boost::none, pop(2));
}

TEST_F(QueueMapValueTest, AfterPushingOnePoppingOnePerKey) {
  push(2, 3);
  pop(2);
  EXPECT_EQ(boost::none, pop());
  EXPECT_EQ(boost::none, pop(2));
}

TEST_F(QueueMapValueTest, AfterPushingOnePoppingOnePushingOne) {
  push(2, 3);
  pop();
  push(3, 4);
  EXPECT_EQ(boost::none, pop(2));
  EXPECT_EQ(4, pop(3).value());
  EXPECT_EQ(boost::none, pop());
}

TEST_F(QueueMapValueTest, AfterPushingOnePoppingOnePerKeyPushingOne) {
  push(2, 3);
  pop(2);
  push(3, 4);
  EXPECT_EQ(boost::none, pop(2));
  EXPECT_EQ(4, pop(3).value());
  EXPECT_EQ(boost::none, pop());
}

TEST_F(QueueMapValueTest, PushingSomePoppingMiddlePerKey) {
  push(1, 2);
  push(2, 3);
  push(3, 4);
  push(4, 5);
  push(5, 6);
  EXPECT_EQ(3, pop(2).value());
  EXPECT_EQ(5, pop(4).value());
  EXPECT_EQ(2, pop().value());
  EXPECT_EQ(4, pop().value());
  EXPECT_EQ(6, pop().value());
  EXPECT_EQ(boost::none, pop());
}

TEST_F(QueueMapValueTest, PushingSomePoppingFirstPerKey) {
  push(1, 2);
  push(2, 3);
  push(3, 4);
  push(4, 5);
  push(5, 6);
  EXPECT_EQ(2, pop(1).value());
  EXPECT_EQ(3, pop(2).value());
  EXPECT_EQ(4, pop().value());
  EXPECT_EQ(5, pop().value());
  EXPECT_EQ(6, pop().value());
  EXPECT_EQ(boost::none, pop());
}

TEST_F(QueueMapValueTest, PushingSomePoppingLastPerKey) {
  push(1, 2);
  push(2, 3);
  push(3, 4);
  push(4, 5);
  push(5, 6);
  EXPECT_EQ(6, pop(5).value());
  EXPECT_EQ(5, pop(4).value());
  EXPECT_EQ(2, pop().value());
  EXPECT_EQ(3, pop().value());
  EXPECT_EQ(4, pop().value());
  EXPECT_EQ(boost::none, pop());
}

class QueueMapPeekTest: public QueueMapTest {};

TEST_F(QueueMapPeekTest, PoppingFromEmpty) {
  EXPECT_EQ(boost::none, peek());
}

TEST_F(QueueMapPeekTest, PushingOne) {
  push(3, 2);
  EXPECT_EQ(2, peek().value());
}

TEST_F(QueueMapPeekTest, PushingTwo) {
  push(2, 3);
  push(3, 4);
  EXPECT_EQ(3, peek().value());
  EXPECT_EQ(3, peek().value());
  EXPECT_EQ(3, pop().value());
  EXPECT_EQ(4, peek().value());
  EXPECT_EQ(4, peek().value());
  EXPECT_EQ(4, pop().value());
  EXPECT_EQ(boost::none, peek());
  EXPECT_EQ(boost::none, pop());
}

TEST_F(QueueMapPeekTest, AfterPushingTwoAndPoppingFirst) {
  push(2, 3);
  push(3, 4);
  pop(2);
  EXPECT_EQ(boost::none, pop(2));
  EXPECT_EQ(4, peek().value());
}

class CopyableValueType {
public:
  static int numCopyConstructorCalled;
  CopyableValueType(int value): _value(value) {}
  CopyableValueType(const CopyableValueType &rhs): CopyableValueType(rhs._value) {
    ++numCopyConstructorCalled;
  }
  CopyableValueType(CopyableValueType &&rhs): CopyableValueType(rhs._value) {
    //Don't increase numCopyConstructorCalled
  }
  int value() const {
    return _value;
  }
private:
  int _value;
};
int CopyableValueType::numCopyConstructorCalled = 0;

//Test that QueueMap uses a move constructor for Value if possible
class QueueMapMoveConstructorTest: public Test {
public:
  QueueMapMoveConstructorTest() {
    CopyableValueType::numCopyConstructorCalled = 0;
    map = make_unique<QueueMap<MinimalKeyType, CopyableValueType>>();
  }
  unique_ptr<QueueMap<MinimalKeyType, CopyableValueType>> map;
};

TEST_F(QueueMapMoveConstructorTest, PushingAndPopping_MoveIntoMap) {
  map->push(MinimalKeyType::create(0), CopyableValueType(2));
  CopyableValueType val = map->pop().value();
  EXPECT_EQ(0, CopyableValueType::numCopyConstructorCalled);
}

TEST_F(QueueMapMoveConstructorTest, PushingAndPoppingPerKey_MoveIntoMap) {
  map->push(MinimalKeyType::create(0), CopyableValueType(2));
  CopyableValueType val = map->pop(MinimalKeyType::create(0)).value();
  EXPECT_EQ(0, CopyableValueType::numCopyConstructorCalled);
}

TEST_F(QueueMapMoveConstructorTest, PushingAndPopping_CopyIntoMap) {
  CopyableValueType value(2);
  map->push(MinimalKeyType::create(0), value);
  CopyableValueType val = map->pop().value();
  EXPECT_EQ(1, CopyableValueType::numCopyConstructorCalled);
}

TEST_F(QueueMapMoveConstructorTest, PushingAndPoppingPerKey_CopyIntoMap) {
  CopyableValueType value(2);
  map->push(MinimalKeyType::create(0), value);
  CopyableValueType val = map->pop(MinimalKeyType::create(0)).value();
  EXPECT_EQ(1, CopyableValueType::numCopyConstructorCalled);
}

//TODO Pushing the same key twice
