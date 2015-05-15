#include "testutils/QueueMapTest.h"

class QueueMapTest_Size: public QueueMapTest {};

TEST_F(QueueMapTest_Size, Empty) {
  EXPECT_EQ(0, size());
}

TEST_F(QueueMapTest_Size, AfterPushingOne) {
  push(2, 3);
  EXPECT_EQ(1, size());
}

TEST_F(QueueMapTest_Size, AfterPushingTwo) {
  push(2, 3);
  push(3, 4);
  EXPECT_EQ(2, size());
}

TEST_F(QueueMapTest_Size, AfterPushingTwoAndPoppingOldest) {
  push(2, 3);
  push(3, 4);
  pop();
  EXPECT_EQ(1, size());
}

TEST_F(QueueMapTest_Size, AfterPushingTwoAndPoppingFirst) {
  push(2, 3);
  push(3, 4);
  pop(2);
  EXPECT_EQ(1, size());
}

TEST_F(QueueMapTest_Size, AfterPushingTwoAndPoppingLast) {
  push(2, 3);
  push(3, 4);
  pop(3);
  EXPECT_EQ(1, size());
}

TEST_F(QueueMapTest_Size, AfterPushingOnePoppingOne) {
  push(2, 3);
  pop();
  EXPECT_EQ(0, size());
}

TEST_F(QueueMapTest_Size, AfterPushingOnePoppingOnePerKey) {
  push(2, 3);
  pop(2);
  EXPECT_EQ(0, size());
}

TEST_F(QueueMapTest_Size, AfterPushingOnePoppingOnePushingOne) {
  push(2, 3);
  pop();
  push(3, 4);
  EXPECT_EQ(1, size());
}

TEST_F(QueueMapTest_Size, AfterPushingOnePoppingOnePerKeyPushingOne) {
  push(2, 3);
  pop(2);
  push(3, 4);
  EXPECT_EQ(1, size());
}

TEST_F(QueueMapTest_Size, AfterPushingOnePoppingOnePushingSame) {
  push(2, 3);
  pop();
  push(2, 3);
  EXPECT_EQ(1, size());
}

TEST_F(QueueMapTest_Size, AfterPushingOnePoppingOnePerKeyPushingSame) {
  push(2, 3);
  pop(2);
  push(2, 3);
  EXPECT_EQ(1, size());
}
