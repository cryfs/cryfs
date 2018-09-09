#include <gtest/gtest.h>
#include "cpp-utils/pointer/unique_ref.h"
#include <vector>
#include <set>
#include <map>
#include <unordered_set>
#include <unordered_map>

using namespace cpputils;

namespace {
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
struct SomeBaseClass {
  SomeBaseClass(int v_): v(v_) {}
  int v;
};
struct SomeChildClass : SomeBaseClass {
  SomeChildClass(int v): SomeBaseClass(v) {}
};
}

static_assert(std::is_same<SomeClass, unique_ref<SomeClass>::element_type>::value, "unique_ref<T>::element_type is wrong");
static_assert(std::is_same<int, unique_ref<int, SomeClass1Parameter>::element_type>::value, "unique_ref<T,D>::element_type is wrong");
static_assert(std::is_same<SomeClass1Parameter, unique_ref<int, SomeClass1Parameter>::deleter_type>::value, "unique_ref<T,D>::deleter_type is wrong");

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

TEST(MakeUniqueRefTest, CanAssignToUniquePtr) {
  std::unique_ptr<int> var = make_unique_ref<int>(2);
  EXPECT_EQ(2, *var);
}

TEST(MakeUniqueRefTest, CanAssignToSharedPtr) {
  std::shared_ptr<int> var = make_unique_ref<int>(2);
  EXPECT_EQ(2, *var);
 }

 TEST(MakeUniqueRefTest, CanAssignToBaseClassPtr) {
  unique_ref<SomeBaseClass> var = make_unique_ref<SomeChildClass>(3);
  EXPECT_EQ(3, var->v);
}

TEST(MakeUniqueRefTest, CanAssignToBaseClassUniquePtr) {
  std::unique_ptr<SomeBaseClass> var = make_unique_ref<SomeChildClass>(3);
  EXPECT_EQ(3, var->v);
}

TEST(MakeUniqueRefTest, CanAssignToBaseClassSharedPtr) {
  std::shared_ptr<SomeBaseClass> var = make_unique_ref<SomeChildClass>(3);
  EXPECT_EQ(3, var->v);
}

TEST(NullcheckTest, givenUniquePtrToInt_withNullptr_whenNullcheckCalled_thenReturnsNone) {
  boost::optional<unique_ref<int>> var = nullcheck(std::unique_ptr<int>(nullptr));
  EXPECT_FALSE(static_cast<bool>(var));
}

TEST(NullcheckTest, givenUniquePtrToObject_withNullptr_whenNullcheckCalled_thenReturnsNone) {
  boost::optional<unique_ref<SomeClass0Parameters>> var = nullcheck(std::unique_ptr<SomeClass0Parameters>(nullptr));
  EXPECT_FALSE(static_cast<bool>(var));
}

TEST(NullcheckTest, givenUniquePtrToInt_withNonNullptr_whenNullcheckCalled_thenReturnsUniqueRef) {
  boost::optional<unique_ref<int>> var = nullcheck(std::make_unique<int>(3));
  EXPECT_TRUE(static_cast<bool>(var));
  EXPECT_EQ(3, **var);
}

TEST(NullcheckTest, givenUniquePtrToObject_withNonNullptr_whenNullcheckCalled_thenReturnsUniqueRef) {
  boost::optional<unique_ref<SomeClass0Parameters>> var = nullcheck(std::make_unique<SomeClass0Parameters>());
  EXPECT_TRUE(static_cast<bool>(var));
  //Check that the type is correct
  EXPECT_EQ(var->get(), dynamic_cast<SomeClass0Parameters*>(var->get()));
}

TEST(NullcheckTest, givenUniquePtrToObjectWith1Parameter_withNonNullptr_whenNullcheckCalled_thenReturnsUniqueRef) {
  boost::optional<unique_ref<SomeClass1Parameter>> var = nullcheck(std::make_unique<SomeClass1Parameter>(5));
  EXPECT_TRUE(static_cast<bool>(var));
  EXPECT_EQ(5, (*var)->param);
}

TEST(NullcheckTest, givenUniquePtrToObjectWith2Parameters_withNonNullptr_whenNullcheckCalled_thenReturnsUniqueRef) {
  boost::optional<unique_ref<SomeClass2Parameters>> var = nullcheck(std::make_unique<SomeClass2Parameters>(7,2));
  EXPECT_TRUE(static_cast<bool>(var));
  EXPECT_EQ(7, (*var)->param1);
  EXPECT_EQ(2, (*var)->param2);
}

TEST(NullcheckTest, givenUniquePtrToInt_withNonNullptr_whenNullcheckCalled_thenCanExtractUniqueRef) {
  boost::optional<unique_ref<int>> var = nullcheck(std::make_unique<int>(3));
  unique_ref<int> resolved = std::move(var).value();
}

TEST(NullcheckTest, givenUniquePtrToObject_withNonNullptr_whenNullcheckCalled_thenCanExtractUniqueRef) {
  boost::optional<unique_ref<SomeClass0Parameters>> var = nullcheck(std::make_unique<SomeClass>());
  unique_ref<SomeClass0Parameters> resolved = std::move(var).value();
}

TEST(NullcheckTest, givenUniquePtrToInt_whenCallingNullcheck_thenTypesCanBeAutoDeduced) {
  auto var = nullcheck(std::make_unique<int>(3));
  auto resolved = std::move(var).value();
}

TEST(NullcheckTest, givenUniquePtrToObject_whenCallingNullcheck_thenTypesCanBeAutoDeduced) {
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

TEST_F(UniqueRefTest, givenUniqueRefToInt_whenCallingGet_thenReturnsValue) {
  unique_ref<int> obj = make_unique_ref<int>(3);
  EXPECT_EQ(3, *obj.get());
}

TEST_F(UniqueRefTest, givenUniqueRefToObject_whenCallingGet_thenReturnsObject) {
  unique_ref<SomeClass1Parameter> obj = make_unique_ref<SomeClass1Parameter>(5);
  EXPECT_EQ(5, obj.get()->param);
}

TEST_F(UniqueRefTest, givenUniqueRefToInt_whenDereferencing_thenReturnsValue) {
  unique_ref<int> obj = make_unique_ref<int>(3);
  EXPECT_EQ(3, *obj);
}

TEST_F(UniqueRefTest, givenUniqueRefToObject_whenDereferencing_thenReturnsObject) {
  unique_ref<SomeClass1Parameter> obj = make_unique_ref<SomeClass1Parameter>(5);
  EXPECT_EQ(5, (*obj).param);
}

TEST_F(UniqueRefTest, givenUniqueRefToObject_whenArrowDereferencing_thenReturnsObject) {
  unique_ref<SomeClass1Parameter> obj = make_unique_ref<SomeClass1Parameter>(3);
  EXPECT_EQ(3, obj->param);
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveAssigning_thenPointsToSameObject) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  unique_ref<SomeClass> obj2 = make_unique_ref<SomeClass>();
  SomeClass *obj1ptr = obj1.get();
  obj2 = std::move(obj1);
  EXPECT_EQ(obj1ptr, obj2.get());
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveAssigning_thenOldInstanceInvalid) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  unique_ref<SomeClass> obj2 = make_unique_ref<SomeClass>();
  obj2 = std::move(obj1);
  EXPECT_FALSE(obj1.is_valid()); // NOLINT (intentional use-after-move)
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveAssigningToBaseClass_thenPointsToSameObject) {
  unique_ref<SomeChildClass> child = make_unique_ref<SomeChildClass>(3);
  unique_ref<SomeBaseClass> base = make_unique_ref<SomeBaseClass>(10);
  base = std::move(child);
  EXPECT_EQ(3, base->v); // NOLINT (intentional use-after-move)
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveAssigningToBaseClass_thenOldInstanceInvalid) {
  unique_ref<SomeChildClass> obj1 = make_unique_ref<SomeChildClass>(3);
  unique_ref<SomeBaseClass> obj2 = make_unique_ref<SomeBaseClass>(10);
  obj2 = std::move(obj1);
  EXPECT_FALSE(obj1.is_valid()); // NOLINT (intentional use-after-move)
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveAssigningToUniquePtr_thenPointsToSameObject) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  std::unique_ptr<SomeClass> obj2 = std::make_unique<SomeClass>();
  SomeClass *obj1ptr = obj1.get();
  obj2 = std::move(obj1);
  EXPECT_EQ(obj1ptr, obj2.get());
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveAssigningToUniquePtr_thenOldInstanceInvalid) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  std::unique_ptr<SomeClass> obj2 = std::make_unique<SomeClass>();
  obj2 = std::move(obj1);
  EXPECT_FALSE(obj1.is_valid()); // NOLINT (intentional use-after-move)
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveAssigningToBaseClassUniquePtr_thenPointsToSameObject) {
  unique_ref<SomeChildClass> child = make_unique_ref<SomeChildClass>(3);
  std::unique_ptr<SomeBaseClass> base = std::make_unique<SomeBaseClass>(10);
  base = std::move(child);
  EXPECT_EQ(3, base->v);
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveAssigningToBaseClassUniquePtr_thenOldInstanceInvalid) {
  unique_ref<SomeChildClass> obj1 = make_unique_ref<SomeChildClass>(3);
  std::unique_ptr<SomeBaseClass> obj2 = std::make_unique<SomeBaseClass>(10);
  obj2 = std::move(obj1);
  EXPECT_FALSE(obj1.is_valid()); // NOLINT (intentional use-after-move)
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveAssigningToSharedPtr_thenPointsToSameObject) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  std::shared_ptr<SomeClass> obj2 = std::make_shared<SomeClass>();
  SomeClass *obj1ptr = obj1.get();
  obj2 = std::move(obj1);
  EXPECT_EQ(obj1ptr, obj2.get());
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveAssigningToSharedPtr_thenOldInstanceInvalid) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  std::shared_ptr<SomeClass> obj2 = std::make_shared<SomeClass>();
  obj2 = std::move(obj1);
  EXPECT_FALSE(obj1.is_valid()); // NOLINT (intentional use-after-move)
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveAssigningToBaseClassSharedPtr_thenPointsToSameObject) {
  unique_ref<SomeChildClass> child = make_unique_ref<SomeChildClass>(3);
  std::shared_ptr<SomeBaseClass> base = std::make_shared<SomeBaseClass>(10);
  base = std::move(child);
  EXPECT_EQ(3, base->v);
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveAssigningToBaseClassSharedPtr_thenOldInstanceInvalid) {
  unique_ref<SomeChildClass> obj1 = make_unique_ref<SomeChildClass>(3);
  std::shared_ptr<SomeBaseClass> obj2 = std::make_shared<SomeBaseClass>(10);
  obj2 = std::move(obj1);
  EXPECT_FALSE(obj1.is_valid()); // NOLINT (intentional use-after-move)
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveConstructing_thenPointsToSameObject) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  SomeClass *obj1ptr = obj1.get();
  unique_ref<SomeClass> obj2 = std::move(obj1);
  EXPECT_EQ(obj1ptr, obj2.get());
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveConstructing_thenOldInstanceInvalid) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  unique_ref<SomeClass> obj2 = std::move(obj1);
  EXPECT_FALSE(obj1.is_valid()); // NOLINT (intentional use-after-move)
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveConstructingToBaseClass_thenPointsToSameObject) {
  unique_ref<SomeChildClass> child = make_unique_ref<SomeChildClass>(3);
  unique_ref<SomeBaseClass> base = std::move(child);
  EXPECT_EQ(3, base->v);
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveConstructingToBaseClass_thenOldInstanceInvalid) {
  unique_ref<SomeChildClass> child = make_unique_ref<SomeChildClass>(3);
  unique_ref<SomeBaseClass> base = std::move(child);
  EXPECT_FALSE(child.is_valid()); // NOLINT (intentional use-after-move)
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveConstructingToUniquePtr_thenPointsToSameObject) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  SomeClass *obj1ptr = obj1.get();
  std::unique_ptr<SomeClass> obj2 = std::move(obj1);
  EXPECT_EQ(obj1ptr, obj2.get());
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveConstructingToUniquePtr_thenOldInstanceInvalid) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  std::unique_ptr<SomeClass> obj2 = std::move(obj1);
  EXPECT_FALSE(obj1.is_valid()); // NOLINT (intentional use-after-move)
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveConstructingToBaseClassUniquePtr_thenPointsToSameObject) {
  unique_ref<SomeChildClass> child = make_unique_ref<SomeChildClass>(3);
  std::unique_ptr<SomeBaseClass> base = std::move(child);
  EXPECT_EQ(3, base->v);
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveConstructingToBaseClassUniquePtr_thenOldInstanceInvalid) {
  unique_ref<SomeChildClass> child = make_unique_ref<SomeChildClass>(3);
  std::unique_ptr<SomeBaseClass> base = std::move(child);
  EXPECT_FALSE(child.is_valid()); // NOLINT (intentional use-after-move)
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveConstructingToSharedPtr_thenPointsToSameObject) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  SomeClass *obj1ptr = obj1.get();
  std::shared_ptr<SomeClass> obj2 = std::move(obj1);
  EXPECT_EQ(obj1ptr, obj2.get());
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveConstructingToSharedPtr_thenOldInstanceInvalid) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  std::shared_ptr<SomeClass> obj2 = std::move(obj1);
  EXPECT_FALSE(obj1.is_valid()); // NOLINT (intentional use-after-move)
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveConstructingToBaseClassSharedPtr_thenPointsToSameObject) {
  unique_ref<SomeChildClass> child = make_unique_ref<SomeChildClass>(3);
  std::shared_ptr<SomeBaseClass> base = std::move(child);
  EXPECT_EQ(3, base->v);
}

TEST_F(UniqueRefTest, givenUniqueRef_whenMoveConstructingToBaseClassSharedPtr_thenOldInstanceInvalid) {
  unique_ref<SomeChildClass> child = make_unique_ref<SomeChildClass>(3);
  std::shared_ptr<SomeBaseClass> base = std::move(child);
  EXPECT_FALSE(child.is_valid()); // NOLINT (intentional use-after-move)
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
  EXPECT_TRUE(obj1.is_valid());
  EXPECT_FALSE(obj2.is_valid());
}

TEST_F(UniqueRefTest, SwapWithInvalid) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  unique_ref<SomeClass> obj2 = make_unique_ref<SomeClass>();
  makeInvalid(std::move(obj2));
  SomeClass *obj1ptr = obj1.get();
  std::swap(obj1, obj2);
  EXPECT_FALSE(obj1.is_valid());
  EXPECT_TRUE(obj2.is_valid());
  EXPECT_EQ(obj1ptr, obj2.get());
}

TEST_F(UniqueRefTest, SwapInvalidWithInvalid) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  unique_ref<SomeClass> obj2 = make_unique_ref<SomeClass>();
  makeInvalid(std::move(obj1));
  makeInvalid(std::move(obj2));
  std::swap(obj1, obj2);
  EXPECT_FALSE(obj1.is_valid());
  EXPECT_FALSE(obj2.is_valid());
}

TEST_F(UniqueRefTest, SwapFromRValue) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  unique_ref<SomeClass> obj2 = make_unique_ref<SomeClass>();
  SomeClass *obj1ptr = obj1.get();
  SomeClass *obj2ptr = obj2.get();
  std::swap(std::move(obj1), obj2);
  EXPECT_EQ(obj2ptr, obj1.get()); // NOLINT (intentional use-after-move)
  EXPECT_EQ(obj1ptr, obj2.get());
}

TEST_F(UniqueRefTest, SwapWithRValue) {
  unique_ref<SomeClass> obj1 = make_unique_ref<SomeClass>();
  unique_ref<SomeClass> obj2 = make_unique_ref<SomeClass>();
  SomeClass *obj1ptr = obj1.get();
  SomeClass *obj2ptr = obj2.get();
  std::swap(obj1, std::move(obj2));
  EXPECT_EQ(obj2ptr, obj1.get());
  EXPECT_EQ(obj1ptr, obj2.get()); // NOLINT (intentional use-after-move)
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
  EXPECT_TRUE(var1 == var2); // NOLINT (intentional use-after-move)
  EXPECT_FALSE(var1 != var2); // NOLINT (intentional use-after-move)
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
  EXPECT_TRUE(var1 != var2); // NOLINT (intentional use-after-move)
  EXPECT_FALSE(var1 == var2); // NOLINT (intentional use-after-move)
}

TEST_F(UniqueRefTest, Nonequality_NullptrRight) {
  unique_ref<int> var1 = make_unique_ref<int>(3);
  unique_ref<int> var2 = make_unique_ref<int>(3);
  makeInvalid(std::move(var2));
  EXPECT_TRUE(var1 != var2); // NOLINT (intentional use-after-move)
  EXPECT_FALSE(var1 == var2); // NOLINT (intentional use-after-move)
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
  EXPECT_NE(std::hash<unique_ref<int>>()(var1), std::hash<unique_ref<int>>()(var2)); // NOLINT (intentional use-after-move)
}

TEST_F(UniqueRefTest, HashIsDifferent_NullptrRight) {
  unique_ref<int> var1 = make_unique_ref<int>(3);
  unique_ref<int> var2 = make_unique_ref<int>(3);
  makeInvalid(std::move(var2));
  EXPECT_NE(std::hash<unique_ref<int>>()(var1), std::hash<unique_ref<int>>()(var2)); // NOLINT (intentional use-after-move)
}

TEST_F(UniqueRefTest, HashIsSame_BothNullptr) {
  unique_ref<int> var1 = make_unique_ref<int>(3);
  unique_ref<int> var2 = make_unique_ref<int>(3);
  makeInvalid(std::move(var1));
  makeInvalid(std::move(var2));
  EXPECT_EQ(std::hash<unique_ref<int>>()(var1), std::hash<unique_ref<int>>()(var2)); // NOLINT (intentional use-after-move)
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
  EXPECT_TRUE(std::less<unique_ref<int>>()(var1, var2)); // NOLINT (intentional use-after-move)
}

TEST_F(UniqueRefTest, NullptrIsLess2) {
  unique_ref<int> var1 = make_unique_ref<int>(3);
  unique_ref<int> var2 = make_unique_ref<int>(3);
  makeInvalid(std::move(var2));
  EXPECT_FALSE(std::less<unique_ref<int>>()(var1, var2)); // NOLINT (intentional use-after-move)
}

TEST_F(UniqueRefTest, NullptrIsNotLessThanNullptr) {
  unique_ref<int> var1 = make_unique_ref<int>(3);
  unique_ref<int> var2 = make_unique_ref<int>(3);
  makeInvalid(std::move(var1));
  makeInvalid(std::move(var2));
  EXPECT_FALSE(std::less<unique_ref<int>>()(var1, var2)); // NOLINT (intentional use-after-move)
}

namespace {
class OnlyMoveable {
public:
  OnlyMoveable(int value_): value(value_)  {}
  OnlyMoveable(OnlyMoveable &&source) noexcept: value(source.value) {source.value = -1;}
  bool operator==(const OnlyMoveable &rhs) const {
    return value == rhs.value;
  }
  int value;
private:
  OnlyMoveable(const OnlyMoveable& rhs) = delete;
  OnlyMoveable& operator=(const OnlyMoveable& rhs) = delete;
};
}

TEST_F(UniqueRefTest, AllowsDerefOnRvalue) {
  OnlyMoveable val = *make_unique_ref<OnlyMoveable>(5);
  EXPECT_EQ(OnlyMoveable(5), val);
}

namespace {
class DestructableMock final {
public:
  DestructableMock(bool* wasDestructed): wasDestructed_(wasDestructed) {}

  ~DestructableMock() {
    *wasDestructed_ = true;
  }

private:
  bool* wasDestructed_;
};
}

TEST_F(UniqueRefTest, givenUniqueRefWithDefaultDeleter_whenDestructed_thenCallsDefaultDeleter) {
  bool wasDestructed = false;
  {
    auto obj = make_unique_ref<DestructableMock>(&wasDestructed);
    EXPECT_FALSE(wasDestructed);
  }
  EXPECT_TRUE(wasDestructed);
}

TEST_F(UniqueRefTest, givenUniqueRefWithDefaultDeleter_whenMoveConstructed_thenCallsDefaultDeleterAfterSecondDestructed) {
  bool wasDestructed = false;
  auto obj = make_unique_ref<DestructableMock>(&wasDestructed);
  {
    unique_ref<DestructableMock> obj2 = std::move(obj);
    EXPECT_FALSE(wasDestructed);
  }
  EXPECT_TRUE(wasDestructed);
}

TEST_F(UniqueRefTest, givenUniqueRefWithDefaultDeleter_whenMoveAssigned_thenCallDefaultDeleterAfterSecondDestructed) {
  bool dummy = false;
  bool wasDestructed = false;
  unique_ref<DestructableMock> obj = make_unique_ref<DestructableMock>(&wasDestructed);
  {
    unique_ref<DestructableMock> obj2 = make_unique_ref<DestructableMock>(&dummy);
    obj2 = std::move(obj);
    EXPECT_FALSE(wasDestructed);
  }
  EXPECT_TRUE(wasDestructed);
}

TEST_F(UniqueRefTest, givenUniqueRefWithDefaultDeleter_whenDestructCalled_thenCallsDefaultDeleter) {
  bool wasDestructed = false;
  auto obj = make_unique_ref<DestructableMock>(&wasDestructed);
  destruct(std::move(obj));
  EXPECT_TRUE(wasDestructed);
  EXPECT_FALSE(obj.is_valid()); // NOLINT (intentional use-after-move)
}

namespace {
struct SetToTrueDeleter final {
  void operator()(bool* ptr) {
    *ptr = true;
  }
};
}

TEST_F(UniqueRefTest, givenUniqueRefWithCustomDefaultConstructibleDeleter_whenDestructed_thenCallsCustomDeleter) {
  bool wasDestructed = false;
  {
    auto obj = nullcheck(std::unique_ptr<bool, SetToTrueDeleter>(&wasDestructed)).value();
    EXPECT_FALSE(wasDestructed);
  }
  EXPECT_TRUE(wasDestructed);
}

TEST_F(UniqueRefTest, givenUniqueRefWithCustomDefaultConstructibleDeleter_whenMoveConstructed_thenCallsCustomDeleterAfterSecondDestructed) {
  bool wasDestructed = false;
  unique_ref<bool, SetToTrueDeleter> obj = nullcheck(std::unique_ptr<bool, SetToTrueDeleter>(&wasDestructed)).value();
  {
    unique_ref<bool, SetToTrueDeleter> obj2 = std::move(obj);
    EXPECT_FALSE(wasDestructed);
  }
  EXPECT_TRUE(wasDestructed);
}

TEST_F(UniqueRefTest, givenUniqueRefWithCustomDefaultConstructibleDeleter_whenMoveAssigned_thenCallsCustomDeleterAfterSecondDestructed) {
  bool dummy = false;
  bool wasDestructed = false;
  unique_ref<bool, SetToTrueDeleter> obj = nullcheck(std::unique_ptr<bool, SetToTrueDeleter>(&wasDestructed)).value();
  {
    unique_ref<bool, SetToTrueDeleter> obj2 = nullcheck(std::unique_ptr<bool, SetToTrueDeleter>(&dummy)).value();
    obj2 = std::move(obj);
    EXPECT_FALSE(wasDestructed);
  }
  EXPECT_TRUE(wasDestructed);
}

TEST_F(UniqueRefTest, givenUniqueRefWithCustomDefaultConstructibleDeleter_whenDestructCalled_thenCallsCustomDeleter) {
  bool wasDestructed = false;
  auto obj = nullcheck(std::unique_ptr<bool, SetToTrueDeleter>(&wasDestructed)).value();
  destruct(std::move(obj));
  EXPECT_TRUE(wasDestructed);
  EXPECT_FALSE(obj.is_valid()); // NOLINT (intentional use-after-move)
}

namespace {
struct SetToDeleter final {
  SetToDeleter(int value): value_(value) {}
  int value_;

  void operator()(int* ptr) {
    *ptr = value_;
  }
};
}

TEST_F(UniqueRefTest, givenUniqueRefWithCustomDeleterInstance_whenDestructed_thenCallsCustomDeleterInstance) {
  int value = 0;
  {
    auto obj = nullcheck(std::unique_ptr<int, SetToDeleter>(&value, SetToDeleter(4))).value();
    EXPECT_EQ(0, value);
  }
  EXPECT_EQ(4, value);
}

TEST_F(UniqueRefTest, givenUniqueRefWithCustomDeleterInstance_whenMoveConstructed_thenCallsCustomDeleterInstanceAfterSecondDestructed) {
  int value = 0;
  unique_ref<int, SetToDeleter> obj = nullcheck(std::unique_ptr<int, SetToDeleter>(&value, SetToDeleter(4))).value();
  {
    unique_ref<int, SetToDeleter> obj2 = std::move(obj);
    EXPECT_EQ(0, value);
  }
  EXPECT_EQ(4, value);
}

TEST_F(UniqueRefTest, givenUniqueRefWithCustomDeleterInstance_whenMoveAssigned_thenCallsCustomDeleterInstanceAfterSecondDestructed) {
  int dummy = 0;
  int value = 0;
  unique_ref<int, SetToDeleter> obj = nullcheck(std::unique_ptr<int, SetToDeleter>(&value, SetToDeleter(4))).value();
  {
    unique_ref<int, SetToDeleter> obj2 = nullcheck(std::unique_ptr<int, SetToDeleter>(&dummy, SetToDeleter(0))).value();
    obj2 = std::move(obj);
    EXPECT_EQ(0, value);
  }
  EXPECT_EQ(4, value);
}

TEST_F(UniqueRefTest, givenUniqueRefWithCustomDeleterInstance_whenDestructCalled_thenCallsCustomDeleterInstance) {
  int value = 0;
  auto obj = nullcheck(std::unique_ptr<int, SetToDeleter>(&value, SetToDeleter(4))).value();
  destruct(std::move(obj));
  EXPECT_EQ(4, value);
  EXPECT_FALSE(obj.is_valid()); // NOLINT (intentional use-after-move)
}

TEST_F(UniqueRefTest, givenUniquePtrWithCustomDeleterInstance_whenMovedToUniquePtr_thenHasSameDeleterInstance) {
  int dummy = 0;
  SetToDeleter deleter(4);
  auto ptr = std::unique_ptr<int, SetToDeleter>(&dummy, deleter);
  auto ref = nullcheck(std::move(ptr)).value();
  EXPECT_EQ(4, ref.get_deleter().value_);
}

TEST_F(UniqueRefTest, givenUniqueRefWithCustomDeleterInstance_whenMoveConstructing_thenHasSameDeleterInstance) {
  int dummy = 0;
  SetToDeleter deleter(4);
  auto ref = nullcheck(std::unique_ptr<int, SetToDeleter>(&dummy, deleter)).value();
  unique_ref<int, SetToDeleter> ref2 = std::move(ref);
  EXPECT_EQ(4, ref2.get_deleter().value_);
}

TEST_F(UniqueRefTest, givenUniqueRefWithCustomDeleterInstance_whenMoveAssigning_thenHasSameDeleterInstance) {
  int dummy = 0;
  SetToDeleter deleter(4);
  auto ref = nullcheck(std::unique_ptr<int, SetToDeleter>(&dummy, deleter)).value();
  auto ref2 = nullcheck(std::unique_ptr<int, SetToDeleter>(&dummy, SetToDeleter(0))).value();
  ref2 = std::move(ref);
  EXPECT_EQ(4, ref2.get_deleter().value_);
}

TEST_F(UniqueRefTest, AllowsMoveConstructingToUniqueRefOfConst) {
  unique_ref<int> a = make_unique_ref<int>(3);
  unique_ref<const int> b = std::move(a);
}

TEST_F(UniqueRefTest, AllowsMoveAssigningToUniqueRefOfConst) {
  unique_ref<int> a = make_unique_ref<int>(3);
  unique_ref<const int> b = make_unique_ref<int>(10);
  b = std::move(a);
}
