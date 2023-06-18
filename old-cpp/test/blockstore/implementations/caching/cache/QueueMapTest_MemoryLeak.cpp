#include "testutils/QueueMapTest.h"

// Tests that QueueMap calls destructors correctly.
// This is needed, because QueueMap does its own memory management.
class QueueMapTest_MemoryLeak: public QueueMapTest {
public:
  void EXPECT_NUM_INSTANCES(int num) {
    EXPECT_EQ(num, MinimalKeyType::instances);
    EXPECT_EQ(num, MinimalValueType::instances);
  }
};

TEST_F(QueueMapTest_MemoryLeak, Empty) {
  EXPECT_NUM_INSTANCES(0);
}

TEST_F(QueueMapTest_MemoryLeak, AfterPushingOne) {
  push(2, 3);
  EXPECT_NUM_INSTANCES(1);
}

TEST_F(QueueMapTest_MemoryLeak, AfterPushingTwo) {
  push(2, 3);
  push(3, 4);
  EXPECT_NUM_INSTANCES(2);
}

TEST_F(QueueMapTest_MemoryLeak, AfterPushingTwoAndPoppingOldest) {
  push(2, 3);
  push(3, 4);
  pop();
  EXPECT_NUM_INSTANCES(1);
}

TEST_F(QueueMapTest_MemoryLeak, AfterPushingTwoAndPoppingFirst) {
  push(2, 3);
  push(3, 4);
  pop(2);
  EXPECT_NUM_INSTANCES(1);
}

TEST_F(QueueMapTest_MemoryLeak, AfterPushingTwoAndPoppingLast) {
  push(2, 3);
  push(3, 4);
  pop(3);
  EXPECT_NUM_INSTANCES(1);
}

TEST_F(QueueMapTest_MemoryLeak, AfterPushingOnePoppingOne) {
  push(2, 3);
  pop();
  EXPECT_NUM_INSTANCES(0);
}

TEST_F(QueueMapTest_MemoryLeak, AfterPushingOnePoppingOnePerKey) {
  push(2, 3);
  pop(2);
  EXPECT_NUM_INSTANCES(0);
}

TEST_F(QueueMapTest_MemoryLeak, AfterPushingOnePoppingOnePushingOne) {
  push(2, 3);
  pop();
  push(3, 4);
  EXPECT_NUM_INSTANCES(1);
}

TEST_F(QueueMapTest_MemoryLeak, AfterPushingOnePoppingOnePerKeyPushingOne) {
  push(2, 3);
  pop(2);
  push(3, 4);
  EXPECT_NUM_INSTANCES(1);
}

TEST_F(QueueMapTest_MemoryLeak, AfterPushingOnePoppingOnePushingSame) {
  push(2, 3);
  pop();
  push(2, 3);
  EXPECT_NUM_INSTANCES(1);
}

TEST_F(QueueMapTest_MemoryLeak, AfterPushingOnePoppingOnePerKeyPushingSame) {
  push(2, 3);
  pop(2);
  push(2, 3);
  EXPECT_NUM_INSTANCES(1);
}
