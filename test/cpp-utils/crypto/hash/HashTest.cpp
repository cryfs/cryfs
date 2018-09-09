#include <gtest/gtest.h>
#include <cpp-utils/crypto/hash/Hash.h>
#include <cpp-utils/data/DataFixture.h>

using namespace cpputils::hash;
using cpputils::DataFixture;
using cpputils::Data;

TEST(HashTest, generateSalt_isIndeterministic) {
  EXPECT_NE(generateSalt(), generateSalt());
}

TEST(HashTest, hash_setsSaltCorrectly) {
  Salt salt = generateSalt();
  Data data = DataFixture::generate(1024);
  EXPECT_EQ(salt, hash(data, salt).salt);
}

TEST(HashTest, hash_isDeterministicWithSameDataSameSalt) {
  Salt salt = generateSalt();
  Data data = DataFixture::generate(1024);
  EXPECT_EQ(hash(data, salt).digest, hash(data, salt).digest);
}

TEST(HashTest, hash_isIndeterministicWithSameDataDifferentSalt) {
  Salt salt1 = generateSalt();
  Salt salt2 = generateSalt();
  Data data = DataFixture::generate(1024);
  EXPECT_NE(hash(data, salt1).digest, hash(data, salt2).digest);
}

TEST(HashTest, hash_isIndeterministicWithDifferentDataSameSalt) {
  Salt salt = generateSalt();
  Data data1 = DataFixture::generate(1024, 1);
  Data data2 = DataFixture::generate(1024, 2);
  EXPECT_NE(hash(data1, salt).digest, hash(data2, salt).digest);
}

TEST(HashTest, hash_isIndeterministicWithDifferentDataDifferentSalt) {
  Salt salt1 = generateSalt();
  Salt salt2 = generateSalt();
  Data data1 = DataFixture::generate(1024, 1);
  Data data2 = DataFixture::generate(1024, 2);
  EXPECT_NE(hash(data1, salt1).digest, hash(data2, salt2).digest);
}
