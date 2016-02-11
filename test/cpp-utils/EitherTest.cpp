#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <boost/optional/optional_io.hpp>
#include "cpp-utils/either.h"
#include "cpp-utils/macros.h"
#include <sstream>

//TODO Go through all test cases and think about whether it makes sense to add the same test case but with primitive types.

using std::ostringstream;
using std::string;
using std::vector;
using std::pair;
using std::make_pair;
using namespace cpputils;
using ::testing::Test;

class OnlyMoveable {
public:
  OnlyMoveable(int value_): value(value_)  {}
  OnlyMoveable(OnlyMoveable &&source): value(source.value) {source.value = -1;}
  bool operator==(const OnlyMoveable &rhs) const {
    return value == rhs.value;
  }
  int value;
private:
  DISALLOW_COPY_AND_ASSIGN(OnlyMoveable);
};

template<typename T>
struct StoreWith1ByteFlag {
  T val;
  char flag;
};

class EitherTest: public Test {
public:
  template<class Left, class Right>
  void EXPECT_IS_LEFT(const either<Left,Right> &val) {
    EXPECT_TRUE(val.is_left());
    EXPECT_FALSE(val.is_right());
  }
  template<class Left, class Right>
  void EXPECT_IS_RIGHT(const either<Left,Right> &val) {
    EXPECT_FALSE(val.is_left());
    EXPECT_TRUE(val.is_right());
  }
  template<class Left, class Right, class Expected>
  void EXPECT_LEFT_IS(const Expected &expected, either<Left, Right> &value) {
    EXPECT_IS_LEFT(value);
    EXPECT_EQ(expected, value.left());
    EXPECT_EQ(expected, value.left_opt().get());
    EXPECT_EQ(boost::none, value.right_opt());
    const either<Left, Right> &const_value = value;
    EXPECT_EQ(expected, const_value.left());
    EXPECT_EQ(expected, const_value.left_opt().get());
    EXPECT_EQ(boost::none, const_value.right_opt());
  }
  template<class Left, class Right, class Expected>
  void EXPECT_RIGHT_IS(const Expected &expected, either<Left, Right> &value) {
    EXPECT_IS_RIGHT(value);
    EXPECT_EQ(expected, value.right());
    EXPECT_EQ(expected, value.right_opt().get());
    EXPECT_EQ(boost::none, value.left_opt());
    const either<Left, Right> &const_value = value;
    EXPECT_EQ(expected, const_value.right());
    EXPECT_EQ(expected, const_value.right_opt().get());
    EXPECT_EQ(boost::none, const_value.left_opt());
  }
};

template<typename Left, typename Right>
void TestSpaceUsage() {
  EXPECT_EQ(std::max(sizeof(StoreWith1ByteFlag<Left>), sizeof(StoreWith1ByteFlag<Right>)), sizeof(either<Left, Right>));
}

TEST_F(EitherTest, SpaceUsage) {
  TestSpaceUsage<char, int>();
  TestSpaceUsage<int, short>();
  TestSpaceUsage<char, short>();
  TestSpaceUsage<int, string>();
  TestSpaceUsage<string, vector<string>>();
}

TEST_F(EitherTest, LeftCanBeConstructed) {
  either<int, string> val = 3;
  UNUSED(val);
}

TEST_F(EitherTest, RightCanBeConstructed) {
  either<int, string> val = string("string");
  UNUSED(val);
}

TEST_F(EitherTest, IsLeft) {
  either<int, string> val = 3;
  EXPECT_IS_LEFT(val);
}

TEST_F(EitherTest, IsRight) {
  either<int, string> val = string("string");
  EXPECT_IS_RIGHT(val);
}

TEST_F(EitherTest, LeftIsStored) {
  either<int, string> val = 3;
  EXPECT_LEFT_IS(3, val);
}

TEST_F(EitherTest, RightIsStored) {
  either<int, string> val = string("string");
  EXPECT_RIGHT_IS("string", val);
}

TEST_F(EitherTest, LeftCanBeMoveContructed) {
  either<OnlyMoveable, string> val = OnlyMoveable(1);
  UNUSED(val);
}

TEST_F(EitherTest, RightCanBeMoveContructed) {
  either<string, OnlyMoveable> val = OnlyMoveable(1);
  UNUSED(val);
}

TEST_F(EitherTest, IsLeftWhenMoveContructed) {
  either<OnlyMoveable, string> val = OnlyMoveable(1);
  EXPECT_IS_LEFT(val);
}

TEST_F(EitherTest, IsRightWhenMoveContructed) {
  either<string, OnlyMoveable> val = OnlyMoveable(1);
  EXPECT_IS_RIGHT(val);
}

TEST_F(EitherTest, LeftIsStoredWhenMoveContructed) {
  either<OnlyMoveable, string> val = OnlyMoveable(2);
  EXPECT_LEFT_IS(OnlyMoveable(2), val);
}

TEST_F(EitherTest, RightIsStoredWhenMoveContructed) {
  either<string, OnlyMoveable> val = OnlyMoveable(3);
  EXPECT_RIGHT_IS(OnlyMoveable(3), val);
}

TEST_F(EitherTest, LeftCanBeCopied) {
  either<string, int> val = string("string");
  either<string, int> val2 = val;
  EXPECT_LEFT_IS("string", val2);
}

TEST_F(EitherTest, CopyingLeftDoesntChangeSource) {
  either<string, int> val = string("string");
  either<string, int> val2 = val;
  EXPECT_LEFT_IS("string", val);
}

TEST_F(EitherTest, RightCanBeCopied) {
  either<int, string> val = string("string");
  either<int, string> val2 = val;
  EXPECT_RIGHT_IS("string", val2);
}

TEST_F(EitherTest, CopyingRightDoesntChangeSource) {
  either<int, string> val = string("string");
  either<int, string> val2 = val;
  EXPECT_RIGHT_IS("string", val);
}

TEST_F(EitherTest, LeftCanBeMoved) {
  either<OnlyMoveable, int> val = OnlyMoveable(5);
  either<OnlyMoveable, int> val2 = std::move(val);
  EXPECT_LEFT_IS(OnlyMoveable(5), val2);
}

TEST_F(EitherTest, RightCanBeMoved) {
  either<int, OnlyMoveable> val = OnlyMoveable(5);
  either<int, OnlyMoveable> val2 = std::move(val);
  EXPECT_RIGHT_IS(OnlyMoveable(5), val2);
}

TEST_F(EitherTest, LeftCanBeAssigned) {
  either<string, int> val = string("string");
  either<string, int> val2 = string("otherstring");
  val2 = val;
  EXPECT_LEFT_IS("string", val2);
}

TEST_F(EitherTest, RightCanBeAssigned) {
  either<int, string> val = string("string");
  either<int, string> val2 = string("otherstring");
  val2 = val;
  EXPECT_RIGHT_IS("string", val2);
}

TEST_F(EitherTest, LeftCanBeMoveAssigned) {
  either<OnlyMoveable, int> val = OnlyMoveable(3);
  either<OnlyMoveable, int> val2 = OnlyMoveable(4);
  val2 = std::move(val);
  EXPECT_LEFT_IS(OnlyMoveable(3), val2);
}

TEST_F(EitherTest, RightCanBeMoveAssigned) {
  either<int, OnlyMoveable> val = OnlyMoveable(3);
  either<int, OnlyMoveable> val2 = OnlyMoveable(4);
  val2 = std::move(val);
  EXPECT_RIGHT_IS(OnlyMoveable(3), val2);
}

TEST_F(EitherTest, LeftCanBeDirectlyAssigned) {
  either<string, int> val = string("string");
  val = string("otherstring");
  EXPECT_LEFT_IS("otherstring", val);
}

TEST_F(EitherTest, RightCanBeDirectlyAssigned) {
  either<int, string> val = string("string");
  val = string("otherstring");
  EXPECT_RIGHT_IS("otherstring", val);
}

TEST_F(EitherTest, LeftCanBeDirectlyMoveAssigned) {
  either<OnlyMoveable, int> val = OnlyMoveable(3);
  val = OnlyMoveable(5);
  EXPECT_LEFT_IS(OnlyMoveable(5), val);
}

TEST_F(EitherTest, RightCanBeDirectlyMoveAssigned) {
  either<int, OnlyMoveable> val = OnlyMoveable(3);
  val = OnlyMoveable(5);
  EXPECT_RIGHT_IS(OnlyMoveable(5), val);
}

TEST_F(EitherTest, ModifyLeft) {
  either<string, int> val = string("mystring1");
  val.left() = "mystring2";
  EXPECT_LEFT_IS("mystring2", val);
}

TEST_F(EitherTest, ModifyRight) {
  either<int, string> val = string("mystring1");
  val.right() = "mystring2";
  EXPECT_RIGHT_IS("mystring2", val);
}

TEST_F(EitherTest, ModifyLeftOpt) {
  either<string, int> val = string("mystring1");
  val.left_opt().get() = "mystring2";
  EXPECT_LEFT_IS("mystring2", val);
}

TEST_F(EitherTest, ModifyRightOpt) {
  either<int, string> val = string("mystring1");
  val.right_opt().get() = "mystring2";
  EXPECT_RIGHT_IS("mystring2", val);
}

TEST_F(EitherTest, LeftEquals) {
  either<string, int> val1 = string("mystring");
  either<string, int> val2 = string("mystring");
  EXPECT_TRUE(val1 == val2);
  EXPECT_TRUE(val2 == val1);
  EXPECT_FALSE(val1 != val2);
  EXPECT_FALSE(val2 != val1);
}

TEST_F(EitherTest, LeftNotEquals) {
  either<string, int> val1 = string("mystring");
  either<string, int> val2 = string("mystring2");
  EXPECT_TRUE(val1 != val2);
  EXPECT_TRUE(val2 != val1);
  EXPECT_FALSE(val1 == val2);
  EXPECT_FALSE(val2 == val1);
}

TEST_F(EitherTest, RightEquals) {
  either<int, string> val1 = string("mystring");
  either<int, string> val2 = string("mystring");
  EXPECT_TRUE(val1 == val2);
  EXPECT_TRUE(val2 == val1);
  EXPECT_FALSE(val1 != val2);
  EXPECT_FALSE(val2 != val1);
}

TEST_F(EitherTest, RightNotEquals) {
  either<int, string> val1 = string("mystring");
  either<int, string> val2 = string("mystring2");
  EXPECT_TRUE(val1 != val2);
  EXPECT_TRUE(val2 != val1);
  EXPECT_FALSE(val1 == val2);
  EXPECT_FALSE(val2 == val1);
}

TEST_F(EitherTest, LeftNotEqualsRight) {
  either<string, int> val1 = string("mystring");
  either<string, int> val2 = 3;
  EXPECT_TRUE(val1 != val2);
  EXPECT_TRUE(val2 != val1);
  EXPECT_FALSE(val1 == val2);
  EXPECT_FALSE(val2 == val1);
}

TEST_F(EitherTest, OutputLeft) {
  ostringstream str;
  str << either<string, int>("mystring");
  EXPECT_EQ("Left(mystring)", str.str());
}

TEST_F(EitherTest, OutputRight) {
  ostringstream str;
  str << either<int, string>("mystring");
  EXPECT_EQ("Right(mystring)", str.str());
}

TEST_F(EitherTest, MakeLeft) {
  either<string, int> var = make_left<string, int>("mystring");
  EXPECT_LEFT_IS("mystring", var);
}

TEST_F(EitherTest, MakeLeft_OnlyMoveable) {
  either<OnlyMoveable, int> var = make_left<OnlyMoveable, int>(4);
  EXPECT_LEFT_IS(OnlyMoveable(4), var);
}

TEST_F(EitherTest, MakeLeft_MultiParam) {
  either<pair<int, int>, int> var = make_left<pair<int, int>, int>(4, 5);
  EXPECT_LEFT_IS(make_pair(4,5), var);
}

TEST_F(EitherTest, MakeRight) {
  either<int, string> var = make_right<int, string>("mystring");
  EXPECT_RIGHT_IS("mystring", var);
}

TEST_F(EitherTest, MakeRight_OnlyMoveable) {
  either<int, OnlyMoveable> var = make_right<int, OnlyMoveable>(4);
  EXPECT_RIGHT_IS(OnlyMoveable(4), var);
}

TEST_F(EitherTest, MakeRight_MultiParam) {
  either<int, pair<int, int>> var = make_right<int, pair<int, int>>(4, 5);
  EXPECT_RIGHT_IS(make_pair(4,5), var);
}

TEST_F(EitherTest, LeftCanBeQueriedAsRvalue) {
  OnlyMoveable val = make_left<OnlyMoveable, int>(3).left();
  EXPECT_EQ(OnlyMoveable(3), val);
}

TEST_F(EitherTest, RightCanBeQueriedAsRvalue) {
  OnlyMoveable val = make_right<int, OnlyMoveable>(3).right();
  EXPECT_EQ(OnlyMoveable(3), val);
}

TEST_F(EitherTest, LeftOptCanBeQueriedAsRvalue) {
  OnlyMoveable val = make_left<OnlyMoveable, int>(3).left_opt().value();
  EXPECT_EQ(OnlyMoveable(3), val);
}

TEST_F(EitherTest, RightOptCanBeQueriedAsRvalue) {
  OnlyMoveable val = make_right<int, OnlyMoveable>(3).right_opt().value();
  EXPECT_EQ(OnlyMoveable(3), val);
}

class DestructorCallback {
public:
  MOCK_CONST_METHOD0(call, void());

  void EXPECT_CALLED(int times = 1) {
    EXPECT_CALL(*this, call()).Times(times);
  }
};
class ClassWithDestructorCallback {
public:
  ClassWithDestructorCallback(const DestructorCallback *destructorCallback) : _destructorCallback(destructorCallback) {}
  ClassWithDestructorCallback(const ClassWithDestructorCallback &rhs): _destructorCallback(rhs._destructorCallback) {}

  ~ClassWithDestructorCallback() {
    _destructorCallback->call();
  }

private:
  const DestructorCallback *_destructorCallback;

  ClassWithDestructorCallback &operator=(const ClassWithDestructorCallback &rhs) = delete;
};
class OnlyMoveableClassWithDestructorCallback {
public:
  OnlyMoveableClassWithDestructorCallback(const DestructorCallback *destructorCallback) : _destructorCallback(destructorCallback) { }
  OnlyMoveableClassWithDestructorCallback(OnlyMoveableClassWithDestructorCallback &&source): _destructorCallback(source._destructorCallback) {}

  ~OnlyMoveableClassWithDestructorCallback() {
    _destructorCallback->call();
  }

private:
  DISALLOW_COPY_AND_ASSIGN(OnlyMoveableClassWithDestructorCallback);
  const DestructorCallback *_destructorCallback;
};

class EitherTest_Destructor: public EitherTest {
};

TEST_F(EitherTest_Destructor, LeftDestructorIsCalled) {
  DestructorCallback destructorCallback;
  destructorCallback.EXPECT_CALLED(2);  //Once for the temp object, once when the either class destructs

  ClassWithDestructorCallback temp(&destructorCallback);
  either<ClassWithDestructorCallback, string> var = temp;
}

TEST_F(EitherTest_Destructor, RightDestructorIsCalled) {
  DestructorCallback destructorCallback;
  destructorCallback.EXPECT_CALLED(2);  //Once for the temp object, once when the either class destructs

  ClassWithDestructorCallback temp(&destructorCallback);
  either<string, ClassWithDestructorCallback> var = temp;
}

TEST_F(EitherTest_Destructor, LeftDestructorIsCalledAfterCopying) {
  DestructorCallback destructorCallback;
  destructorCallback.EXPECT_CALLED(3);  //Once for the temp object, once for var1 and once for var2

  ClassWithDestructorCallback temp(&destructorCallback);
  either<ClassWithDestructorCallback, string> var1 = temp;
  either<ClassWithDestructorCallback, string> var2 = var1;
}

TEST_F(EitherTest_Destructor, RightDestructorIsCalledAfterCopying) {
  DestructorCallback destructorCallback;
  destructorCallback.EXPECT_CALLED(3);  //Once for the temp object, once for var1 and once for var2

  ClassWithDestructorCallback temp(&destructorCallback);
  either<string, ClassWithDestructorCallback> var1 = temp;
  either<string, ClassWithDestructorCallback> var2 = var1;
}

TEST_F(EitherTest_Destructor, LeftDestructorIsCalledAfterMoving) {
  DestructorCallback destructorCallback;
  destructorCallback.EXPECT_CALLED(3);  //Once for the temp object, once for var1 and once for var2

  OnlyMoveableClassWithDestructorCallback temp(&destructorCallback);
  either<OnlyMoveableClassWithDestructorCallback, string> var1 = std::move(temp);
  either<OnlyMoveableClassWithDestructorCallback, string> var2 = std::move(var1);
}

TEST_F(EitherTest_Destructor, RightDestructorIsCalledAfterMoving) {
  DestructorCallback destructorCallback;
  destructorCallback.EXPECT_CALLED(3);  //Once for the temp object, once for var1 and once for var2

  OnlyMoveableClassWithDestructorCallback temp(&destructorCallback);
  either<string, OnlyMoveableClassWithDestructorCallback> var1 = std::move(temp);
  either<string, OnlyMoveableClassWithDestructorCallback> var2 = std::move(var1);
}

TEST_F(EitherTest_Destructor, LeftDestructorIsCalledAfterAssignment) {
  DestructorCallback destructorCallback1;
  DestructorCallback destructorCallback2;
  destructorCallback1.EXPECT_CALLED(2); //Once for the temp1 object, once at the assignment
  destructorCallback2.EXPECT_CALLED(3); //Once for the temp2 object, once in destructor of var2, once in destructor of var1

  ClassWithDestructorCallback temp1(&destructorCallback1);
  either<ClassWithDestructorCallback, string> var1 = temp1;
  ClassWithDestructorCallback temp2(&destructorCallback2);
  either<ClassWithDestructorCallback, string> var2 = temp2;
  var1 = var2;
}

TEST_F(EitherTest_Destructor, RightDestructorIsCalledAfterAssignment) {
  DestructorCallback destructorCallback1;
  DestructorCallback destructorCallback2;
  destructorCallback1.EXPECT_CALLED(2); //Once for the temp1 object, once at the assignment
  destructorCallback2.EXPECT_CALLED(3); //Once for the temp2 object, once in destructor of var2, once in destructor of var1

  ClassWithDestructorCallback temp1(&destructorCallback1);
  either<string, ClassWithDestructorCallback> var1 = temp1;
  ClassWithDestructorCallback temp2(&destructorCallback2);
  either<string, ClassWithDestructorCallback> var2 = temp2;
  var1 = var2;
}

TEST_F(EitherTest_Destructor, LeftDestructorIsCalledAfterMoveAssignment) {
  DestructorCallback destructorCallback1;
  DestructorCallback destructorCallback2;
  destructorCallback1.EXPECT_CALLED(2); //Once for the temp1 object, once at the assignment
  destructorCallback2.EXPECT_CALLED(3); //Once for the temp2 object, once in destructor of var2, once in destructor of var1

  OnlyMoveableClassWithDestructorCallback temp1(&destructorCallback1);
  either<OnlyMoveableClassWithDestructorCallback, string> var1 = std::move(temp1);
  OnlyMoveableClassWithDestructorCallback temp2(&destructorCallback2);
  either<OnlyMoveableClassWithDestructorCallback, string> var2 = std::move(temp2);
  var1 = std::move(var2);
}

TEST_F(EitherTest_Destructor, RightDestructorIsCalledAfterMoveAssignment) {
  DestructorCallback destructorCallback1;
  DestructorCallback destructorCallback2;
  destructorCallback1.EXPECT_CALLED(2); //Once for the temp1 object, once at the assignment
  destructorCallback2.EXPECT_CALLED(3); //Once for the temp2 object, once in destructor of var2, once in destructor of var1

  OnlyMoveableClassWithDestructorCallback temp1(&destructorCallback1);
  either<string, OnlyMoveableClassWithDestructorCallback> var1 = std::move(temp1);
  OnlyMoveableClassWithDestructorCallback temp2(&destructorCallback2);
  either<string, OnlyMoveableClassWithDestructorCallback> var2 = std::move(temp2);
  var1 = std::move(var2);
}
