#include <gtest/gtest.h>

#include "fspp/impl/IdList.h"
#include <stdexcept>

using cpputils::make_unique_ref;

using namespace fspp;

class MyObj {
public:
  MyObj(int val_): val(val_) {}
  int val;
};

struct IdListTest: public ::testing::Test {
  static constexpr int OBJ1 = 3;
  static constexpr int OBJ2 = 10;
  static constexpr int OBJ3 = 8;

  IdListTest(): list() {}

  IdList<MyObj> list;

  int add(int num) {
    return list.add(make_unique_ref<MyObj>(num));
  }
  int add() {
    return add(OBJ1);
  }
  void check(int id, int num) {
    EXPECT_EQ(num, list.get(id)->val);
  }
  void checkConst(int id, int num) {
    const IdList<MyObj> &constList = list;
    EXPECT_EQ(num, constList.get(id)->val);
  }
};

TEST_F(IdListTest, EmptyList1) {
  ASSERT_THROW(list.get(0), std::out_of_range);
}

TEST_F(IdListTest, EmptyList2) {
  ASSERT_THROW(list.get(3), std::out_of_range);
}

TEST_F(IdListTest, InvalidId) {
  const int valid_id = add();
  const int invalid_id = valid_id + 1;
  ASSERT_THROW(list.get(invalid_id), std::out_of_range);
}

TEST_F(IdListTest, GetRemovedItemOnEmptyList) {
  const int id = add();
  list.remove(id);
  ASSERT_THROW(list.get(id), std::out_of_range);
}

TEST_F(IdListTest, GetRemovedItemOnNonEmptyList) {
  const int id = add();
  add();
  list.remove(id);
  ASSERT_THROW(list.get(id), std::out_of_range);
}

TEST_F(IdListTest, RemoveOnEmptyList1) {
  ASSERT_THROW(list.remove(0), std::out_of_range);
}

TEST_F(IdListTest, RemoveOnEmptyList2) {
  ASSERT_THROW(list.remove(4), std::out_of_range);
}

TEST_F(IdListTest, RemoveInvalidId) {
  const int valid_id = add();
  const int invalid_id = valid_id + 1;
  ASSERT_THROW(list.remove(invalid_id), std::out_of_range);
}

TEST_F(IdListTest, Add1AndGet) {
  const int id = add(OBJ1);
  check(id, OBJ1);
}

TEST_F(IdListTest, Add2AndGet) {
  const int id1 = add(OBJ1);
  const int id2 = add(OBJ2);
  check(id1, OBJ1);
  check(id2, OBJ2);
}

TEST_F(IdListTest, Add3AndGet) {
  const int id1 = add(OBJ1);
  const int id2 = add(OBJ2);
  const int id3 = add(OBJ3);
  check(id1, OBJ1);
  check(id3, OBJ3);
  check(id2, OBJ2);
}

TEST_F(IdListTest, Add3AndConstGet) {
  const int id1 = add(OBJ1);
  const int id2 = add(OBJ2);
  const int id3 = add(OBJ3);
  checkConst(id1, OBJ1);
  checkConst(id3, OBJ3);
  checkConst(id2, OBJ2);
}
