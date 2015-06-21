#include "google/gtest/gtest.h"
#include "google/gmock/gmock.h"
#include "../../pointer/cast.h"

using namespace cpputils;

using std::unique_ptr;
using std::make_unique;
using std::function;

// Putting them in an own namespace is needed, so they don't clash with globally defined Parent/Child classes
namespace withdestructorcallback {
    class DestructorCallback {
    public:
        MOCK_CONST_METHOD0(call, void());
    };

    class Parent {
    public:
        virtual ~Parent() { }
    };

    class Child : public Parent {
    public:
        Child(const DestructorCallback &childDestructorCallback) : _destructorCallback(childDestructorCallback) { }

        ~Child() {
            _destructorCallback.call();
        }

    private:
        const DestructorCallback &_destructorCallback;
    };
}
using namespace withdestructorcallback;

class DynamicPointerMoveDestructorTest: public ::testing::Test {
public:
    DestructorCallback childDestructorCallback;
    unique_ptr<Child> createChild() {
        return make_unique<Child>(childDestructorCallback);
    }
    void EXPECT_CHILD_DESTRUCTOR_CALLED() {
        EXPECT_CALL(childDestructorCallback, call()).Times(1);
    }
};

TEST_F(DynamicPointerMoveDestructorTest, ChildInParentPtr) {
  unique_ptr<Parent> parent = createChild();
  EXPECT_CHILD_DESTRUCTOR_CALLED();
}

TEST_F(DynamicPointerMoveDestructorTest, ChildToParentCast) {
  unique_ptr<Child> child = createChild();
  unique_ptr<Parent> parent = dynamic_pointer_move<Parent>(child);
  EXPECT_CHILD_DESTRUCTOR_CALLED();
}

TEST_F(DynamicPointerMoveDestructorTest, ParentToChildCast) {
  unique_ptr<Parent> parent = createChild();
  unique_ptr<Child> child = dynamic_pointer_move<Child>(parent);
  EXPECT_CHILD_DESTRUCTOR_CALLED();
}
