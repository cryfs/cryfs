#include <google/gtest/gtest.h>
#include "../../../../src/config/crypto/kdf/DerivedKey.h"
#include <messmer/cpp-utils/data/DataFixture.h>

using namespace cryfs;
using cpputils::DataFixture;
using cpputils::Data;

TEST(DerivedKeyTest, Config) {
    DerivedKey<32> key(DerivedKeyConfig(DataFixture::generate(32, 1), 1024, 8, 16), DataFixture::generateFixedSize<32>(2));
    EXPECT_EQ(DataFixture::generate(32, 1), key.config().salt());
    EXPECT_EQ(1024, key.config().N());
    EXPECT_EQ(8, key.config().r());
    EXPECT_EQ(16, key.config().p());
}

TEST(DerivedKeyTest, Key) {
    DerivedKey<32> key(DerivedKeyConfig(DataFixture::generate(32, 1), 1024, 8, 16), DataFixture::generateFixedSize<32>(2));
    EXPECT_EQ(DataFixture::generateFixedSize<32>(2), key.key());
}
