#include <google/gtest/gtest.h>
#include "../unique_ref.h"

using namespace cpputils;

//TODO Add some test cases

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
  unique_ref<int> resolved = std::move(*var);
}

TEST(NullcheckTest, OptionIsResolvable_Object) {
  boost::optional<unique_ref<SomeClass0Parameters>> var = nullcheck(std::make_unique<SomeClass0Parameters>());
  unique_ref<SomeClass0Parameters> resolved = std::move(*var);
}

TEST(NullcheckTest, OptionIsAutoResolvable_Primitive) {
  auto var = nullcheck(std::make_unique<int>(3));
  auto resolved = std::move(*var);
}

TEST(NullcheckTest, OptionIsAutoResolvable_Object) {
  auto var = nullcheck(std::make_unique<SomeClass0Parameters>());
  auto resolved = std::move(*var);
}
