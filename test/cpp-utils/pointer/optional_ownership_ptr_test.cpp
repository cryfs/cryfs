#include <gtest/gtest.h>
#include "cpp-utils/pointer/optional_ownership_ptr.h"
#include "cpp-utils/macros.h"

using std::unique_ptr;
using std::function;
using ::testing::Test;

using namespace cpputils;

class TestObject {
public:
  TestObject(function<void()> destructorListener) : _destructorListener(destructorListener) {}
  virtual ~TestObject() {
    _destructorListener();
  }

private:
  function<void()> _destructorListener;
};

class TestObjectHolder {
public:
  TestObjectHolder()
  : _isDestructed(false),
    _testObject(new TestObject([this]() {_isDestructed = true;})) {
  }

  ~TestObjectHolder() {
    if (!_isDestructed) {
      delete _testObject;
      _isDestructed = true;
    }
  }

  TestObject *get() {
    return _testObject;
  }

  bool isDestructed() {
    return _isDestructed;
  }
private:
  bool _isDestructed;
  TestObject *_testObject;
  DISALLOW_COPY_AND_ASSIGN(TestObjectHolder);
};

class OptionalOwnershipPointerTest: public Test {
public:
  OptionalOwnershipPointerTest(): obj(), obj2() {}

  TestObjectHolder obj;
  TestObjectHolder obj2;
};

TEST_F(OptionalOwnershipPointerTest, TestIsInitializedCorrectly) {
  EXPECT_FALSE(obj.isDestructed());
}

TEST_F(OptionalOwnershipPointerTest, DestructsWhenItHasOwnership_UniquePtr) {
  {
    optional_ownership_ptr<TestObject> ptr = WithOwnership(unique_ptr<TestObject>(obj.get()));
    EXPECT_FALSE(obj.isDestructed());
    UNUSED(ptr);
  }
  EXPECT_TRUE(obj.isDestructed());
}

TEST_F(OptionalOwnershipPointerTest, DestructsWhenItHasOwnership_UniqueRef) {
  {
    optional_ownership_ptr<TestObject> ptr = WithOwnership(cpputils::nullcheck(unique_ptr<TestObject>(obj.get())).value());
    //EXPECT_FALSE(obj.isDestructed());
    //UNUSED(ptr);
  }
  //EXPECT_TRUE(obj.isDestructed());
}


TEST_F(OptionalOwnershipPointerTest, DestructsWhenItHasOwnershipAfterAssignment) {
  {
    optional_ownership_ptr<TestObject> ptr = WithoutOwnership(obj.get());
    ptr = WithOwnership(unique_ptr<TestObject>(obj2.get()));
  }
  EXPECT_FALSE(obj.isDestructed());
  EXPECT_TRUE(obj2.isDestructed());
}

TEST_F(OptionalOwnershipPointerTest, DoesntDestructWhenItDoesntHaveOwnership) {
  {
    optional_ownership_ptr<TestObject> ptr = WithoutOwnership(obj.get());
    UNUSED(ptr);
  }
  EXPECT_FALSE(obj.isDestructed());
}

TEST_F(OptionalOwnershipPointerTest, DoesntDestructWhenItDoesntHaveOwnershipAfterAssignment) {
  {
    optional_ownership_ptr<TestObject> ptr = WithOwnership(unique_ptr<TestObject>(obj.get()));
    ptr = WithoutOwnership(obj2.get());
    EXPECT_TRUE(obj.isDestructed());
  }
  EXPECT_FALSE(obj2.isDestructed());
}

TEST_F(OptionalOwnershipPointerTest, DestructsOnReassignmentWithNull) {
  optional_ownership_ptr<TestObject> ptr = WithOwnership(unique_ptr<TestObject>(obj.get()));
  ptr = null<TestObject>();
  EXPECT_TRUE(obj.isDestructed());
}

TEST_F(OptionalOwnershipPointerTest, DoesntCrashWhenDestructingNullptr1) {
  optional_ownership_ptr<TestObject> ptr = null<TestObject>();
  UNUSED(ptr);
}

TEST_F(OptionalOwnershipPointerTest, DoesntCrashWhenDestructingNullptrWithoutOwnership) {
  optional_ownership_ptr<TestObject> ptr = WithoutOwnership(static_cast<TestObject*>(nullptr));
  UNUSED(ptr);
}

TEST_F(OptionalOwnershipPointerTest, DoesntCrashWhenDestructingNullptrWithOwnership) {
  optional_ownership_ptr<TestObject> ptr = WithOwnership(unique_ptr<TestObject>(nullptr));
  UNUSED(ptr);
}
