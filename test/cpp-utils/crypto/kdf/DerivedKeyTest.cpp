#include <gtest/gtest.h>
#include "cpp-utils/crypto/kdf/DerivedKey.h"
#include "cpp-utils/data/DataFixture.h"

using namespace cpputils;

TEST(DerivedKeyTest, Config) {
    DerivedKey<32> key(DerivedKeyConfig(DataFixture::generate(32, 1), 1024, 8, 16), DataFixture::generateFixedSize<32>(2));
    EXPECT_EQ(DataFixture::generate(32, 1), key.config().salt());
    EXPECT_EQ(1024u, key.config().N());
    EXPECT_EQ(8u, key.config().r());
    EXPECT_EQ(16u, key.config().p());
}

TEST(DerivedKeyTest, Key) {
    DerivedKey<32> key(DerivedKeyConfig(DataFixture::generate(32, 1), 1024, 8, 16), DataFixture::generateFixedSize<32>(2));
    EXPECT_EQ(DataFixture::generateFixedSize<32>(2), key.key());
}
