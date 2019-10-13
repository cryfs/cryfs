#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include "cpp-utils/pointer/cast.h"
#include "cpp-utils/pointer/unique_ref.h"
#include "cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h"

//TODO There is a lot of duplication here, because each test case is there twice - once for unique_ptr, once for unique_ref. Remove redundancy by using generic test cases.
//TODO Then also move the unique_ref related test cases there - cast_test.cpp should only contain the unique_ptr related ones.

using namespace cpputils;
using std::unique_ptr;
using std::make_unique;
using boost::optional;
using boost::none;

class DestructorCallback {
public:
  MOCK_METHOD(void, call, (), (const));
};

class Parent {
public:
  virtual ~Parent() { }
};

class Child : public Parent {
public:
  Child(const DestructorCallback *childDestructorCallback) : _destructorCallback(childDestructorCallback) { }
  Child(): Child(nullptr) {}

  ~Child() {
    if (_destructorCallback != nullptr) {
      _destructorCallback->call();
    }
  }

private:
  const DestructorCallback *_destructorCallback;
  DISALLOW_COPY_AND_ASSIGN(Child);
};

class Child2 : public Parent {};


TEST(UniquePtr_DynamicPointerMoveTest, NullPtrParentToChildCast) {
  unique_ptr<Parent> source(nullptr);
  unique_ptr<Child> casted = dynamic_pointer_move<Child>(source);
  EXPECT_EQ(nullptr, source.get());
  EXPECT_EQ(nullptr, casted.get());
}

TEST(UniquePtr_DynamicPointerMoveTest, NullPtrChildToParentCast) {
  unique_ptr<Child> source(nullptr);
  unique_ptr<Parent> casted = dynamic_pointer_move<Parent>(source);
  EXPECT_EQ(nullptr, source.get());
  EXPECT_EQ(nullptr, casted.get());
}

TEST(UniquePtr_DynamicPointerMoveTest, NullPtrSelfCast) {
  unique_ptr<Parent> source(nullptr);
  unique_ptr<Parent> casted = dynamic_pointer_move<Parent>(source);
  EXPECT_EQ(nullptr, source.get());
  EXPECT_EQ(nullptr, casted.get());
}

TEST(UniqueRef_DynamicPointerMoveTest, ValidParentToChildCast) {
  Child *obj = new Child();
  unique_ref<Parent> source(nullcheck(unique_ptr<Parent>(obj)).value());
  unique_ref<Child> casted = dynamic_pointer_move<Child>(source).value();
  EXPECT_FALSE(source.is_valid());  // source lost ownership
  EXPECT_EQ(obj, casted.get());
}

TEST(UniquePtr_DynamicPointerMoveTest, ValidParentToChildCast) {
  Child *obj = new Child();
  unique_ptr<Parent> source(obj);
  unique_ptr<Child> casted = dynamic_pointer_move<Child>(source);
  EXPECT_EQ(nullptr, source.get()); // source lost ownership
  EXPECT_EQ(obj, casted.get());
}

TEST(UniqueRef_DynamicPointerMoveTest, InvalidParentToChildCast1) {
  Parent *obj = new Parent();
  unique_ref<Parent> source(nullcheck(unique_ptr<Parent>(obj)).value());
  optional<unique_ref<Child>> casted = dynamic_pointer_move<Child>(source);
  EXPECT_EQ(obj, source.get()); // source still has ownership
  EXPECT_EQ(none, casted);
}

TEST(UniquePtr_DynamicPointerMoveTest, InvalidParentToChildCast1) {
  Parent *obj = new Parent();
  unique_ptr<Parent> source(obj);
  unique_ptr<Child> casted = dynamic_pointer_move<Child>(source);
  EXPECT_EQ(obj, source.get()); // source still has ownership
  EXPECT_EQ(nullptr, casted.get());
}

TEST(UniqueRef_DynamicPointerMoveTest, InvalidParentToChildCast2) {
  Child2 *obj = new Child2();
  unique_ref<Parent> source(nullcheck(unique_ptr<Parent>(obj)).value());
  optional<unique_ref<Child>> casted = dynamic_pointer_move<Child>(source);
  EXPECT_EQ(obj, source.get()); // source still has ownership
  EXPECT_EQ(none, casted);
}

TEST(UniquePtr_DynamicPointerMoveTest, InvalidParentToChildCast2) {
  Child2 *obj = new Child2();
  unique_ptr<Parent> source(obj);
  unique_ptr<Child> casted = dynamic_pointer_move<Child>(source);
  EXPECT_EQ(obj, source.get()); // source still has ownership
  EXPECT_EQ(nullptr, casted.get());
}

TEST(UniqueRef_DynamicPointerMoveTest, ChildToParentCast) {
  Child *obj = new Child();
  unique_ref<Child> source(nullcheck(unique_ptr<Child>(obj)).value());
  unique_ref<Parent> casted = dynamic_pointer_move<Parent>(source).value();
  EXPECT_FALSE(source.is_valid());  // source lost ownership
  EXPECT_EQ(obj, casted.get());
}

TEST(UniquePtr_DynamicPointerMoveTest, ChildToParentCast) {
  Child *obj = new Child();
  unique_ptr<Child> source(obj);
  unique_ptr<Parent> casted = dynamic_pointer_move<Parent>(source);
  EXPECT_EQ(nullptr, source.get()); // source lost ownership
  EXPECT_EQ(obj, casted.get());
}


class UniqueRef_DynamicPointerMoveDestructorTest: public ::testing::Test {
public:
  UniqueRef_DynamicPointerMoveDestructorTest(): childDestructorCallback() {}

  DestructorCallback childDestructorCallback;
  unique_ref<Child> createChild() {
    return make_unique_ref<Child>(&childDestructorCallback);
  }
  void EXPECT_CHILD_DESTRUCTOR_CALLED() {
    EXPECT_CALL(childDestructorCallback, call()).Times(1);
  }
};

class UniquePtr_DynamicPointerMoveDestructorTest: public ::testing::Test {
public:
  UniquePtr_DynamicPointerMoveDestructorTest(): childDestructorCallback() {}

  DestructorCallback childDestructorCallback;
  unique_ptr<Child> createChild() {
    return make_unique<Child>(&childDestructorCallback);
  }
  void EXPECT_CHILD_DESTRUCTOR_CALLED() {
    EXPECT_CALL(childDestructorCallback, call()).Times(1);
  }
};

TEST_F(UniqueRef_DynamicPointerMoveDestructorTest, ChildInParentPtr) {
  unique_ref<Parent> parent = createChild();
  EXPECT_CHILD_DESTRUCTOR_CALLED();
}

TEST_F(UniquePtr_DynamicPointerMoveDestructorTest, ChildInParentPtr) {
  unique_ptr<Parent> parent = createChild();
  EXPECT_CHILD_DESTRUCTOR_CALLED();
}

TEST_F(UniqueRef_DynamicPointerMoveDestructorTest, ChildToParentCast) {
  unique_ref<Child> child = createChild();
  unique_ref<Parent> parent = dynamic_pointer_move<Parent>(child).value();
  EXPECT_CHILD_DESTRUCTOR_CALLED();
}

TEST_F(UniquePtr_DynamicPointerMoveDestructorTest, ChildToParentCast) {
  unique_ptr<Child> child = createChild();
  unique_ptr<Parent> parent = dynamic_pointer_move<Parent>(child);
  EXPECT_CHILD_DESTRUCTOR_CALLED();
}

TEST_F(UniqueRef_DynamicPointerMoveDestructorTest, ParentToChildCast) {
  unique_ref<Parent> parent = createChild();
  unique_ref<Child> child = dynamic_pointer_move<Child>(parent).value();
  EXPECT_CHILD_DESTRUCTOR_CALLED();
}

TEST_F(UniquePtr_DynamicPointerMoveDestructorTest, ParentToChildCast) {
  unique_ptr<Parent> parent = createChild();
  unique_ptr<Child> child = dynamic_pointer_move<Child>(parent);
  EXPECT_CHILD_DESTRUCTOR_CALLED();
}
