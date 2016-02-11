#include <gtest/gtest.h>
#include "cpp-utils/pointer/unique_ref.h"
#include <vector>
#include <set>
#include <map>
#include <unordered_set>
#include <unordered_map>

using namespace cpputils;

//TODO Test unique_ref destructor
//TODO Test cpputils::destruct()

class SomeClass0Parameters {};
class SomeClass1Parameter {
public:
  SomeClass1Parameter(int param_): param(param_) {}
  int param;
};
class SomeClass2Parameters {
public:
  SomeClass2Parameters(int param1_, int param2_): param1(param1_), param2(param2_) {}
  int param1;
  int param2;
};
using SomeClass = SomeClass0Parameters;

TEST(MakeUniqueRefTest, Primitive) {
  unique_ref<int> var = make_unique_ref<int>(3);
  EXPECT_EQ(3, *var);
}

TEST(MakeUniqueRefTest, ClassWith0Parameters) {
  unique_ref<SomeClass0Parameters> var = make_unique_ref<SomeClass0Parameters>();
  //Check that the type is correct
  EXPECT_EQ(var.get(), dynamic_cast<SomeClass0Parameters*>(var.get()));
}

TEST(MakeUniqueRefTest, ClassWith1Parameter) {
  unique_ref<SomeClass1Parameter> var = make_unique_ref<SomeClass1Parameter>(5);
  EXPECT_EQ(5, var->param);
}

TEST(MakeUniqueRefTest, ClassWith2Parameters) {
  unique_ref<SomeClass2Parameters> var = make_unique_ref<SomeClass2Parameters>(7,2);
  EXPECT_EQ(7, var->param1);
  EXPECT_EQ(2, var->param2);
}

TEST(MakeUniqueRefTest, TypeIsAutoDeductible) {
  auto var1 = make_unique_ref<int>(3);
  auto var2 = make_unique_ref<SomeClass0Parameters>();
  auto var3 = make_unique_ref<SomeClass1Parameter>(2);
  auto var4 = make_unique_ref<SomeClass2Parameters>(2, 3);
}

TEST(NullcheckTest, PrimitiveNullptr) {
  boost::optional<unique_ref<int>> var = nullcheck(std::unique_ptr<int>(nullptr));
  EXPECT_FALSE((bool)var);
}

TEST(NullcheckTest, ObjectNullptr) {
  boost::optional<unique_ref<SomeClass0Parameters>> var = nullcheck(std::unique_ptr<SomeClass0Parameters>(nullptr));
  EXPECT_FALSE((bool)var);
}

TEST(NullcheckTest, Primitive) {
  boost::optional<unique_ref<int>> var = nullcheck(std::make_unique<int>(3));
  EXPECT_TRUE((bool)var);
  EXPECT_EQ(3, **var);
}

TEST(NullcheckTest, ClassWith0Parameters) {
  boost::optional<unique_ref<SomeClass0Parameters>> var = nullcheck(std::make_unique<SomeClass0Parameters>());
  EXPECT_TRUE((bool)var);
  //Check that the type is correct
  EXPECT_EQ(var->get(), dynamic_cast<SomeClass0Parameters*>(var->get()));
}

TEST(NullcheckTest, ClassWith1Parameter) {
  boost::optional<unique_ref<SomeClass1Parameter>> var = nullcheck(std::make_unique<SomeClass1Parameter>(5));
  EXPECT_TRUE((bool)var);
  EXPECT_EQ(5, (*var)->param);
}

TEST(NullcheckTest, ClassWith2Parameters) {
  boost::optional<unique_ref<SomeClass2Parameters>> var = nullcheck(std::make_unique<SomeClass2Parameters>(7,2));
  EXPECT_TRUE((bool)var);
  EXPECT_EQ(7, (*var)->param1);
  EXPECT_EQ(2, (*var)->param2);
}

TEST(NullcheckTest, OptionIsResolvable_Primitive) {
  boost::optional<unique_ref<int>> var = nullcheck(std::make_unique<int>(3));
  unique_ref<int> resolved = std::move(var).value();
}

TEST(NullcheckTest, OptionIsResolvable_Object) {
  boost::optional<unique_ref<SomeClass0Parameters>> var = nullcheck(std::make_unique<SomeClass>());
  unique_ref<SomeClass0Parameters> resolved = std::move(var).value();
}

TEST(NullcheckTest, OptionIsAutoResolvable_Primitive) {
  auto var = nullcheck(std::make_unique<int>(3));
  auto resolved = std::move(var).value();
}

TEST(NullcheckTest, OptionIsAutoResolvable_Object) {
  auto var = nullcheck(std::make_unique<SomeClass>());
  auto resolved = std::move(var).value();
}

class UniqueRefTest: public ::testing::Test {
public:
  template<typename T> void makeInvalid(unique_ref<T> ref) {
    UNUSED(ref);
    //ref is moved in here and then destructed
  }
};

TEST_F(UniqueRefTest, Get_Primitive) {
  unique_ref<int> obj = make_unique_ref<int>(3);
  EXPECT_EQ(3, *obj.get());
}

TEST_F(UniqueRefTest, Get_Object) {
  unique_ref<SomeClass1Parameter> obj = make_unique_ref<SomeClass1Parameter>(5);
  EXPECT_EQ(5, obj.get()->param);
}

TEST_F(UniqueRefTest, Deref_Primitive) {
  unique_ref<int> obj = make_unique_ref<int>(3);
  EXPECT_EQ(3, *obj);
}

TEST_F(UniqueRefTest, Deref_Object) {
  unique_ref<SomeClass1Parameter> obj = make_unique_ref<SomeClass1Parameter>(5);
  EXPECT_EQ(5, (*obj).param);
}

TEST_F(UniqueRefTest, DerefArrow) {
  unique_ref<SomeClass1Parameter> obj = make_unique_ref<SomeClass1Parameter>(3);
  EXPECT_EQ(3, obj->param);
}

TEST_F(UniqueRefTest, Assignment) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  unique_ref<SomeClass> obj2 = make_unique_ref<SomeClass>();
  SomeClass *obj1ptr = obj1.get();
  obj2 = std::move(obj1);
  EXPECT_EQ(obj1ptr, obj2.get());
  EXPECT_EQ(nullptr, obj1.get());
}

TEST_F(UniqueRefTest, MoveConstructor) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  SomeClass *obj1ptr = obj1.get();
  unique_ref<SomeClass> obj2 = std::move(obj1);
  EXPECT_EQ(obj1ptr, obj2.get());
  EXPECT_EQ(nullptr, obj1.get());
}

TEST_F(UniqueRefTest, Swap) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  unique_ref<SomeClass> obj2 = make_unique_ref<SomeClass>();
  SomeClass *obj1ptr = obj1.get();
  SomeClass *obj2ptr = obj2.get();
  std::swap(obj1, obj2);
  EXPECT_EQ(obj2ptr, obj1.get());
  EXPECT_EQ(obj1ptr, obj2.get());
}

TEST_F(UniqueRefTest, SwapFromInvalid) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  makeInvalid(std::move(obj1));
  unique_ref<SomeClass> obj2 = make_unique_ref<SomeClass>();
  SomeClass *obj2ptr = obj2.get();
  std::swap(obj1, obj2);
  EXPECT_EQ(obj2ptr, obj1.get());
  EXPECT_EQ(nullptr, obj2.get());
}

TEST_F(UniqueRefTest, SwapWithInvalid) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  unique_ref<SomeClass> obj2 = make_unique_ref<SomeClass>();
  makeInvalid(std::move(obj2));
  SomeClass *obj1ptr = obj1.get();
  std::swap(obj1, obj2);
  EXPECT_EQ(nullptr, obj1.get());
  EXPECT_EQ(obj1ptr, obj2.get());
}

TEST_F(UniqueRefTest, SwapInvalidWithInvalid) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  unique_ref<SomeClass> obj2 = make_unique_ref<SomeClass>();
  makeInvalid(std::move(obj1));
  makeInvalid(std::move(obj2));
  std::swap(obj1, obj2);
  EXPECT_EQ(nullptr, obj1.get());
  EXPECT_EQ(nullptr, obj2.get());
}

TEST_F(UniqueRefTest, SwapFromRValue) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  unique_ref<SomeClass> obj2 = make_unique_ref<SomeClass>();
  SomeClass *obj1ptr = obj1.get();
  SomeClass *obj2ptr = obj2.get();
  std::swap(std::move(obj1), obj2);
  EXPECT_EQ(obj2ptr, obj1.get());
  EXPECT_EQ(obj1ptr, obj2.get());
}

TEST_F(UniqueRefTest, SwapWithRValue) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  unique_ref<SomeClass> obj2 = make_unique_ref<SomeClass>();
  SomeClass *obj1ptr = obj1.get();
  SomeClass *obj2ptr = obj2.get();
  std::swap(obj1, std::move(obj2));
  EXPECT_EQ(obj2ptr, obj1.get());
  EXPECT_EQ(obj1ptr, obj2.get());
}

TEST_F(UniqueRefTest, CanBePutInContainer_Primitive) {
  std::vector<unique_ref<int>> vec;
  vec.push_back(make_unique_ref<int>(3));
  EXPECT_EQ(3, *vec[0]);
}

TEST_F(UniqueRefTest, CanBePutInContainer_Object) {
  std::vector<unique_ref<SomeClass1Parameter>> vec;
  vec.push_back(make_unique_ref<SomeClass1Parameter>(5));
  EXPECT_EQ(5, vec[0]->param);
}

TEST_F(UniqueRefTest, CanBePutInContainer_Nullcheck) {
  std::vector<unique_ref<int>> vec;
  vec.push_back(*nullcheck(std::make_unique<int>(3)));
  EXPECT_EQ(3, *vec[0]);
}

TEST_F(UniqueRefTest, CanBePutInSet_Primitive) {
  std::set<unique_ref<int>> set;
  set.insert(make_unique_ref<int>(3));
  EXPECT_EQ(3, **set.begin());
}

TEST_F(UniqueRefTest, CanBePutInSet_Object) {
  std::set<unique_ref<SomeClass1Parameter>> set;
  set.insert(make_unique_ref<SomeClass1Parameter>(5));
  EXPECT_EQ(5, (*set.begin())->param);
}

TEST_F(UniqueRefTest, CanBePutInSet_Nullcheck) {
  std::set<unique_ref<int>> set;
  set.insert(*nullcheck(std::make_unique<int>(3)));
  EXPECT_EQ(3, **set.begin());
}

TEST_F(UniqueRefTest, CanBePutInUnorderedSet_Primitive) {
  std::unordered_set<unique_ref<int>> set;
  set.insert(make_unique_ref<int>(3));
  EXPECT_EQ(3, **set.begin());
}

TEST_F(UniqueRefTest, CanBePutInUnorderedSet_Object) {
  std::unordered_set<unique_ref<SomeClass1Parameter>> set;
  set.insert(make_unique_ref<SomeClass1Parameter>(5));
  EXPECT_EQ(5, (*set.begin())->param);
}

TEST_F(UniqueRefTest, CanBePutInUnorderedSet_Nullcheck) {
  std::unordered_set<unique_ref<int>> set;
  set.insert(*nullcheck(std::make_unique<int>(3)));
  EXPECT_EQ(3, **set.begin());
}

TEST_F(UniqueRefTest, CanBePutInMap_Primitive) {
  std::map<unique_ref<int>, unique_ref<int>> map;
  map.insert(std::make_pair(make_unique_ref<int>(3), make_unique_ref<int>(5)));
  EXPECT_EQ(3, *map.begin()->first);
  EXPECT_EQ(5, *map.begin()->second);
}

TEST_F(UniqueRefTest, CanBePutInMap_Object) {
  std::map<unique_ref<SomeClass1Parameter>, unique_ref<SomeClass1Parameter>> map;
  map.insert(std::make_pair(make_unique_ref<SomeClass1Parameter>(5), make_unique_ref<SomeClass1Parameter>(3)));
  EXPECT_EQ(5, map.begin()->first->param);
  EXPECT_EQ(3, map.begin()->second->param);
}

TEST_F(UniqueRefTest, CanBePutInMap_Nullcheck) {
  std::map<unique_ref<int>, unique_ref<int>> map;
  map.insert(std::make_pair(*nullcheck(std::make_unique<int>(3)), *nullcheck(std::make_unique<int>(5))));
  EXPECT_EQ(3, *map.begin()->first);
  EXPECT_EQ(5, *map.begin()->second);
}

TEST_F(UniqueRefTest, CanBePutInUnorderedMap_Primitive) {
  std::unordered_map<unique_ref<int>, unique_ref<int>> map;
  map.insert(std::make_pair(make_unique_ref<int>(3), make_unique_ref<int>(5)));
  EXPECT_EQ(3, *map.begin()->first);
  EXPECT_EQ(5, *map.begin()->second);
}

TEST_F(UniqueRefTest, CanBePutInUnorderedMap_Object) {
  std::unordered_map<unique_ref<SomeClass1Parameter>, unique_ref<SomeClass1Parameter>> map;
  map.insert(std::make_pair(make_unique_ref<SomeClass1Parameter>(5), make_unique_ref<SomeClass1Parameter>(3)));
  EXPECT_EQ(5, map.begin()->first->param);
  EXPECT_EQ(3, map.begin()->second->param);
}

TEST_F(UniqueRefTest, CanBePutInUnorderedMap_Nullcheck) {
  std::unordered_map<unique_ref<int>, unique_ref<int>> map;
  map.insert(std::make_pair(*nullcheck(std::make_unique<int>(3)), *nullcheck(std::make_unique<int>(5))));
  EXPECT_EQ(3, *map.begin()->first);
  EXPECT_EQ(5, *map.begin()->second);
}

TEST_F(UniqueRefTest, Equality_Nullptr) {
  unique_ref<int> var1 = make_unique_ref<int>(3);
  unique_ref<int> var2 = make_unique_ref<int>(4);
  makeInvalid(std::move(var1));
  makeInvalid(std::move(var2));
  EXPECT_TRUE(var1 == var2);
  EXPECT_FALSE(var1 != var2);
}

TEST_F(UniqueRefTest, Nonequality) {
  unique_ref<int> var1 = make_unique_ref<int>(3);
  unique_ref<int> var2 = make_unique_ref<int>(3);
  EXPECT_TRUE(var1 != var2);
  EXPECT_FALSE(var1 == var2);
}

TEST_F(UniqueRefTest, Nonequality_NullptrLeft) {
  unique_ref<int> var1 = make_unique_ref<int>(3);
  unique_ref<int> var2 = make_unique_ref<int>(3);
  makeInvalid(std::move(var1));
  EXPECT_TRUE(var1 != var2);
  EXPECT_FALSE(var1 == var2);
}

TEST_F(UniqueRefTest, Nonequality_NullptrRight) {
  unique_ref<int> var1 = make_unique_ref<int>(3);
  unique_ref<int> var2 = make_unique_ref<int>(3);
  makeInvalid(std::move(var2));
  EXPECT_TRUE(var1 != var2);
  EXPECT_FALSE(var1 == var2);
}

TEST_F(UniqueRefTest, HashIsDifferent) {
  unique_ref<int> var1 = make_unique_ref<int>(3);
  unique_ref<int> var2 = make_unique_ref<int>(3);
  EXPECT_NE(std::hash<unique_ref<int>>()(var1), std::hash<unique_ref<int>>()(var2));
}

TEST_F(UniqueRefTest, HashIsDifferent_NullptrLeft) {
  unique_ref<int> var1 = make_unique_ref<int>(3);
  unique_ref<int> var2 = make_unique_ref<int>(3);
  makeInvalid(std::move(var1));
  EXPECT_NE(std::hash<unique_ref<int>>()(var1), std::hash<unique_ref<int>>()(var2));
}

TEST_F(UniqueRefTest, HashIsDifferent_NullptrRight) {
  unique_ref<int> var1 = make_unique_ref<int>(3);
  unique_ref<int> var2 = make_unique_ref<int>(3);
  makeInvalid(std::move(var2));
  EXPECT_NE(std::hash<unique_ref<int>>()(var1), std::hash<unique_ref<int>>()(var2));
}

TEST_F(UniqueRefTest, HashIsSame_BothNullptr) {
  unique_ref<int> var1 = make_unique_ref<int>(3);
  unique_ref<int> var2 = make_unique_ref<int>(3);
  makeInvalid(std::move(var1));
  makeInvalid(std::move(var2));
  EXPECT_EQ(std::hash<unique_ref<int>>()(var1), std::hash<unique_ref<int>>()(var2));
}

TEST_F(UniqueRefTest, OneIsLess) {
  unique_ref<int> var1 = make_unique_ref<int>(3);
  unique_ref<int> var2 = make_unique_ref<int>(3);
  EXPECT_TRUE(std::less<unique_ref<int>>()(var1, var2) != std::less<unique_ref<int>>()(var2, var1));
}

TEST_F(UniqueRefTest, NullptrIsLess1) {
  unique_ref<int> var1 = make_unique_ref<int>(3);
  unique_ref<int> var2 = make_unique_ref<int>(3);
  makeInvalid(std::move(var1));
  EXPECT_TRUE(std::less<unique_ref<int>>()(var1, var2));
}

TEST_F(UniqueRefTest, NullptrIsLess2) {
  unique_ref<int> var1 = make_unique_ref<int>(3);
  unique_ref<int> var2 = make_unique_ref<int>(3);
  makeInvalid(std::move(var2));
  EXPECT_TRUE(std::less<unique_ref<int>>()(var2, var1));
}

TEST_F(UniqueRefTest, NullptrIsNotLessThanNullptr) {
  unique_ref<int> var1 = make_unique_ref<int>(3);
  unique_ref<int> var2 = make_unique_ref<int>(3);
  makeInvalid(std::move(var1));
  makeInvalid(std::move(var2));
  EXPECT_FALSE(std::less<unique_ref<int>>()(var1, var2));
}

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

TEST_F(UniqueRefTest, AllowsDerefOnRvalue) {
  OnlyMoveable val = *make_unique_ref<OnlyMoveable>(5);
  EXPECT_EQ(OnlyMoveable(5), val);
}