#include "testutils/QueueMapTest.h"
#include <boost/optional/optional_io.hpp>

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
