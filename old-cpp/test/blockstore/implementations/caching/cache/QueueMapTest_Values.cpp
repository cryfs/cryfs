#include "testutils/QueueMapTest.h"
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

class QueueMapTest_Values: public QueueMapTest {};

TEST_F(QueueMapTest_Values, PoppingFromEmpty) {
  EXPECT_EQ(boost::none, pop());
}

TEST_F(QueueMapTest_Values, PoppingFromEmptyPerKey) {
  EXPECT_EQ(boost::none, pop(2));
}

TEST_F(QueueMapTest_Values, PoppingNonexistingPerKey) {
  push(3, 2);
  EXPECT_EQ(boost::none, pop(2));
}

TEST_F(QueueMapTest_Values, PushingOne) {
  push(3, 2);
  EXPECT_EQ(2, pop(3).value());
  EXPECT_EQ(boost::none, pop());
}

TEST_F(QueueMapTest_Values, PushingTwo) {
  push(2, 3);
  push(3, 4);
  EXPECT_EQ(3, pop().value());
  EXPECT_EQ(4, pop().value());
  EXPECT_EQ(boost::none, pop());
}

TEST_F(QueueMapTest_Values, AfterPushingTwoAndPoppingFirst) {
  push(2, 3);
  push(3, 4);
  pop(2);
  EXPECT_EQ(boost::none, pop(2));
  EXPECT_EQ(4, pop(3).value());
  EXPECT_EQ(boost::none, pop());
}

TEST_F(QueueMapTest_Values, AfterPushingTwoAndPoppingLast) {
  push(2, 3);
  push(3, 4);
  pop(3);
  EXPECT_EQ(boost::none, pop(3));
  EXPECT_EQ(3, pop(2).value());
  EXPECT_EQ(boost::none, pop());
}

TEST_F(QueueMapTest_Values, AfterPushingOnePoppingOne) {
  push(2, 3);
  pop();
  EXPECT_EQ(boost::none, pop());
  EXPECT_EQ(boost::none, pop(2));
}

TEST_F(QueueMapTest_Values, AfterPushingOnePoppingOnePerKey) {
  push(2, 3);
  pop(2);
  EXPECT_EQ(boost::none, pop());
  EXPECT_EQ(boost::none, pop(2));
}

TEST_F(QueueMapTest_Values, AfterPushingOnePoppingOnePushingOne) {
  push(2, 3);
  pop();
  push(3, 4);
  EXPECT_EQ(boost::none, pop(2));
  EXPECT_EQ(4, pop(3).value());
  EXPECT_EQ(boost::none, pop());
}

TEST_F(QueueMapTest_Values, AfterPushingOnePoppingOnePerKeyPushingOne) {
  push(2, 3);
  pop(2);
  push(3, 4);
  EXPECT_EQ(boost::none, pop(2));
  EXPECT_EQ(4, pop(3).value());
  EXPECT_EQ(boost::none, pop());
}

TEST_F(QueueMapTest_Values, PushingSomePoppingMiddlePerKey) {
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

TEST_F(QueueMapTest_Values, PushingSomePoppingFirstPerKey) {
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

TEST_F(QueueMapTest_Values, PushingSomePoppingLastPerKey) {
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

//This test forces the underlying datastructure (std::map or std::unordered_map) to grow and reallocate memory.
//So it tests, that QueueMap still works after reallocating memory.
TEST_F(QueueMapTest_Values, ManyValues) {
  //Push 1 million entries
  for (int i = 0; i < 1000000; ++i) {
    push(i, 2*i);
  }
  //pop every other one by key
  for (int i = 0; i < 1000000; i += 2) {
    EXPECT_EQ(2*i, pop(i).value());
  }
  //pop the rest in queue order
  for (int i = 1; i < 1000000; i += 2) {
    EXPECT_EQ(2*i, peek().value());
    EXPECT_EQ(2*i, pop().value());
  }
  EXPECT_EQ(0, size());
  EXPECT_EQ(boost::none, pop());
  EXPECT_EQ(boost::none, peek());
}

TEST_F(QueueMapTest_Values, PushAlreadyExistingValue) {
  push(2, 3);
  EXPECT_ANY_THROW(
    push(2, 4);
  );
}
