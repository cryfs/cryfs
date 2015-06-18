#include "google/gtest/gtest.h"
#include "../pointer.h"

using namespace cpputils;

using std::unique_ptr;

class Parent {
public:
  virtual ~Parent() {}
};
class Child: public Parent {};
class Child2: public Parent {};

//TODO Add test cases that the correct (virtual) destructor is called

TEST(DynamicPointerMoveTest, NullPtrParentToChildCast) {
  unique_ptr<Parent> source(nullptr);
  unique_ptr<Child> casted = dynamic_pointer_move<Child>(source);
  EXPECT_EQ(nullptr, source.get());
  EXPECT_EQ(nullptr, casted.get());
}

TEST(DynamicPointerMoveTest, NullPtrChildToParentCast) {
  unique_ptr<Child> source(nullptr);
  unique_ptr<Parent> casted = dynamic_pointer_move<Parent>(source);
  EXPECT_EQ(nullptr, source.get());
  EXPECT_EQ(nullptr, casted.get());
}

TEST(DynamicPointerMoveTest, NullPtrSelfCast) {
  unique_ptr<Parent> source(nullptr);
  unique_ptr<Parent> casted = dynamic_pointer_move<Parent>(source);
  EXPECT_EQ(nullptr, source.get());
  EXPECT_EQ(nullptr, casted.get());
}

TEST(DynamicPointerMoveTest, ValidParentToChildCast) {
  Child *obj = new Child();
  unique_ptr<Parent> source(obj);
  unique_ptr<Child> casted = dynamic_pointer_move<Child>(source);
  EXPECT_EQ(nullptr, source.get()); // source lost ownership
  EXPECT_EQ(obj, casted.get());
}

TEST(DynamicPointerMoveTest, InvalidParentToChildCast1) {
  Parent *obj = new Parent();
  unique_ptr<Parent> source(obj);
  unique_ptr<Child> casted = dynamic_pointer_move<Child>(source);
  EXPECT_EQ(obj, source.get()); // source still has ownership
  EXPECT_EQ(nullptr, casted.get());
}

TEST(DynamicPointerMoveTest, InvalidParentToChildCast2) {
  Child2 *obj = new Child2();
  unique_ptr<Parent> source(obj);
  unique_ptr<Child> casted = dynamic_pointer_move<Child>(source);
  EXPECT_EQ(obj, source.get()); // source still has ownership
  EXPECT_EQ(nullptr, casted.get());
}

TEST(DynamicPointerMoveTest, ChildToParentCast) {
  Child *obj = new Child();
  unique_ptr<Child> source(obj);
  unique_ptr<Parent> casted = dynamic_pointer_move<Parent>(source);
  EXPECT_EQ(nullptr, source.get()); // source lost ownership
  EXPECT_EQ(obj, casted.get());
}
