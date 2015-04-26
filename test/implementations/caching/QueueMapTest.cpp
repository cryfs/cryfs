#include <google/gtest/gtest.h>
#include <messmer/cpp-utils/macros.h>
#include "../../../implementations/caching/QueueMap.h"

using ::testing::Test;

using namespace blockstore::caching;

// This is a not-default-constructible Key type
class MinimalKeyType {
public:
  static MinimalKeyType create() {
    return MinimalKeyType();
  }
  bool operator==(const MinimalKeyType &rhs) const {
    return true;
  }
private:
  MinimalKeyType() {
  }
};
namespace std {
template <> struct hash<MinimalKeyType> {
  size_t operator()(const MinimalKeyType &obj) const {
    return 0;
  }
};
}
// This is a not-default-constructible non-copyable but moveable Value type
class MinimalValueType {
public:
  static MinimalValueType create() {
    return MinimalValueType();
  }
  MinimalValueType(MinimalValueType &&rhs) = default;
private:
  MinimalValueType() {
  }
  DISALLOW_COPY_AND_ASSIGN(MinimalValueType);
};

class QueueMapTest: public Test {
public:
  QueueMap<int, int> map;
};

TEST_F(QueueMapTest, TypeConstraints) {
  QueueMap<MinimalKeyType, MinimalValueType> obj;
  //Call all functions to ensure they still work
  obj.push(MinimalKeyType::create(), MinimalValueType::create());
  obj.peek();
  obj.pop(MinimalKeyType::create());
  obj.push(MinimalKeyType::create(), MinimalValueType::create());
  obj.pop();
  obj.size();
}

TEST_F(QueueMapTest, Size_Empty) {
  EXPECT_EQ(0, map.size());
}

TEST_F(QueueMapTest, Size_AfterPushingOne) {
  map.push(2, 3);
  EXPECT_EQ(1, map.size());
}

TEST_F(QueueMapTest, Size_AfterPushingTwo) {
  map.push(2, 3);
  map.push(3, 4);
  EXPECT_EQ(2, map.size());
}

TEST_F(QueueMapTest, Size_AfterPushingTwoAndPoppingOldest) {
  map.push(2, 3);
  map.push(3, 4);
  map.pop();
  EXPECT_EQ(1, map.size());
}

TEST_F(QueueMapTest, Size_AfterPushingTwoAndPoppingFirst) {
  map.push(2, 3);
  map.push(3, 4);
  map.pop(2);
  EXPECT_EQ(1, map.size());
}

TEST_F(QueueMapTest, Size_AfterPushingTwoAndPoppingLast) {
  map.push(2, 3);
  map.push(3, 4);
  map.pop(3);
  EXPECT_EQ(1, map.size());
}

TEST_F(QueueMapTest, Size_AfterPushingOnePoppingOne) {
  map.push(2, 3);
  map.pop();
  EXPECT_EQ(0, map.size());
}

TEST_F(QueueMapTest, Size_AfterPushingOnePoppingOnePerKey) {
  map.push(2, 3);
  map.pop(2);
  EXPECT_EQ(0, map.size());
}

TEST_F(QueueMapTest, Size_AfterPushingOnePoppingOnePushingOne) {
  map.push(2, 3);
  map.pop();
  map.push(3, 4);
  EXPECT_EQ(1, map.size());
}

TEST_F(QueueMapTest, Size_AfterPushingOnePoppingOnePerKeyPushingOne) {
  map.push(2, 3);
  map.pop(2);
  map.push(3, 4);
  EXPECT_EQ(1, map.size());
}

TEST_F(QueueMapTest, Size_AfterPushingOnePoppingOnePushingSame) {
  map.push(2, 3);
  map.pop();
  map.push(2, 3);
  EXPECT_EQ(1, map.size());
}

TEST_F(QueueMapTest, Size_AfterPushingOnePoppingOnePerKeyPushingSame) {
  map.push(2, 3);
  map.pop(2);
  map.push(2, 3);
  EXPECT_EQ(1, map.size());
}

//TODO Pushing the same key twice
//TODO Popping from empty
//TODO Popping invalid key
//TODO Test that in all cases, destructors of Value are called correctly in QueueMap when [a] pop() [b] pop(key) [c] ~QueueMap()
//TODO Test that pushing and popping a copy-and-move-constructible object only called 1 copy constructor
//TODO Test that pushing and popping doesn't invalidate objects (e.g, calls too many destructors)
