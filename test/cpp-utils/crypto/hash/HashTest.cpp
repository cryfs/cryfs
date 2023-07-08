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
  const Salt salt = generateSalt();
  const Data data = DataFixture::generate(1024);
  EXPECT_EQ(salt, hash(data, salt).salt);
}

TEST(HashTest, hash_isDeterministicWithSameDataSameSalt) {
  const Salt salt = generateSalt();
  const Data data = DataFixture::generate(1024);
  EXPECT_EQ(hash(data, salt).digest, hash(data, salt).digest);
}

TEST(HashTest, hash_isIndeterministicWithSameDataDifferentSalt) {
  const Salt salt1 = generateSalt();
  const Salt salt2 = generateSalt();
  const Data data = DataFixture::generate(1024);
  EXPECT_NE(hash(data, salt1).digest, hash(data, salt2).digest);
}

TEST(HashTest, hash_isIndeterministicWithDifferentDataSameSalt) {
  const Salt salt = generateSalt();
  const Data data1 = DataFixture::generate(1024, 1);
  const Data data2 = DataFixture::generate(1024, 2);
  EXPECT_NE(hash(data1, salt).digest, hash(data2, salt).digest);
}

TEST(HashTest, hash_isIndeterministicWithDifferentDataDifferentSalt) {
  const Salt salt1 = generateSalt();
  const Salt salt2 = generateSalt();
  const Data data1 = DataFixture::generate(1024, 1);
  const Data data2 = DataFixture::generate(1024, 2);
  EXPECT_NE(hash(data1, salt1).digest, hash(data2, salt2).digest);
}
