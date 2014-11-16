#include "gtest/gtest.h"

#include "fspp/impl/IdList.h"
#include <stdexcept>

using std::make_unique;

using namespace fspp;

class MyObj {
public:
  MyObj(int val_): val(val_) {}
  int val;
};

TEST(IdListTest, EmptyList1) {
  IdList<MyObj> list;
  ASSERT_THROW(list.get(0), std::out_of_range);
}

TEST(IdListTest, EmptyList2) {
  IdList<MyObj> list;
  ASSERT_THROW(list.get(3), std::out_of_range);
}

TEST(IdListTest, InvalidId) {
  IdList<MyObj> list;
  int valid_id = list.add(make_unique<MyObj>(6));
  int invalid_id = valid_id + 1;
  ASSERT_THROW(list.get(invalid_id), std::out_of_range);
}

TEST(IdListTest, GetRemovedItemOnEmptyList) {
  IdList<MyObj> list;
  int id = list.add(make_unique<MyObj>(6));
  list.remove(id);
  ASSERT_THROW(list.get(id), std::out_of_range);
}

TEST(IdListTest, GetRemovedItemOnNonEmptyList) {
  IdList<MyObj> list;
  int id = list.add(make_unique<MyObj>(6));
  list.add(make_unique<MyObj>(5));
  list.remove(id);
  ASSERT_THROW(list.get(id), std::out_of_range);
}

TEST(IdListTest, Add1AndGet) {
  IdList<MyObj> list;
  int id6 = list.add(make_unique<MyObj>(6));
  EXPECT_EQ(6, list.get(id6)->val);
}

TEST(IdListTest, Add2AndGet) {
  IdList<MyObj> list;
  int id4 = list.add(make_unique<MyObj>(4));
  int id5 = list.add(make_unique<MyObj>(5));
  EXPECT_EQ(4, list.get(id4)->val);
  EXPECT_EQ(5, list.get(id5)->val);
}

TEST(IdListTest, Add3AndGet) {
  IdList<MyObj> list;
  int id4 = list.add(make_unique<MyObj>(4));
  int id10 = list.add(make_unique<MyObj>(10));
  int id1 = list.add(make_unique<MyObj>(1));
  EXPECT_EQ(10, list.get(id10)->val);
  EXPECT_EQ(4, list.get(id4)->val);
  EXPECT_EQ(1, list.get(id1)->val);
}

TEST(IdListTest, Add3AndConstGet) {
  IdList<MyObj> list;
  int id4 = list.add(make_unique<MyObj>(4));
  int id10 = list.add(make_unique<MyObj>(10));
  int id1 = list.add(make_unique<MyObj>(1));
  const IdList<MyObj> &const_list = list;
  EXPECT_EQ(10, const_list.get(id10)->val);
  EXPECT_EQ(4, const_list.get(id4)->val);
  EXPECT_EQ(1, const_list.get(id1)->val);
}
