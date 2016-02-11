#include <gtest/gtest.h>
#include "cpp-utils/crypto/kdf/Scrypt.h"

using namespace cpputils;

TEST(SCryptTest, GeneratedKeyIsReproductible_448) {
    auto created = SCrypt().generateKey<56>("mypassword", SCrypt::TestSettings);
    auto recreated = SCrypt().generateKeyFromConfig<56>("mypassword", created.config());
    EXPECT_EQ(created.key(), recreated);
}

TEST(SCryptTest, GeneratedKeyIsReproductible_256) {
    auto created = SCrypt().generateKey<32>("mypassword", SCrypt::TestSettings);
    auto recreated = SCrypt().generateKeyFromConfig<32>("mypassword", created.config());
    EXPECT_EQ(created.key(), recreated);
}

TEST(SCryptTest, GeneratedKeyIsReproductible_128) {
    auto created = SCrypt().generateKey<16>("mypassword", SCrypt::TestSettings);
    auto recreated = SCrypt().generateKeyFromConfig<16>("mypassword", created.config());
    EXPECT_EQ(created.key(), recreated);
}

TEST(SCryptTest, GeneratedKeyIsReproductible_DefaultSettings) {
    auto created = SCrypt().generateKey<16>("mypassword", SCrypt::DefaultSettings);
    auto recreated = SCrypt().generateKeyFromConfig<16>("mypassword", created.config());
    EXPECT_EQ(created.key(), recreated);
}

TEST(SCryptTest, DifferentPasswordResultsInDifferentKey) {
    auto created = SCrypt().generateKey<16>("mypassword", SCrypt::TestSettings);
    auto recreated = SCrypt().generateKeyFromConfig<16>("mypassword2", created.config());
    EXPECT_NE(created.key(), recreated);
}

TEST(SCryptTest, UsesCorrectSettings) {
    auto created = SCrypt().generateKey<16>("mypassword", SCrypt::TestSettings);
    EXPECT_EQ(SCrypt::TestSettings.SALT_LEN, created.config().salt().size());
    EXPECT_EQ(SCrypt::TestSettings.N, created.config().N());
    EXPECT_EQ(SCrypt::TestSettings.r, created.config().r());
    EXPECT_EQ(SCrypt::TestSettings.p, created.config().p());
}

TEST(SCryptTest, UsesCorrectDefaultSettings) {
    auto created = SCrypt().generateKey<16>("mypassword", SCrypt::DefaultSettings);
    EXPECT_EQ(SCrypt::DefaultSettings.SALT_LEN, created.config().salt().size());
    EXPECT_EQ(SCrypt::DefaultSettings.N, created.config().N());
    EXPECT_EQ(SCrypt::DefaultSettings.r, created.config().r());
    EXPECT_EQ(SCrypt::DefaultSettings.p, created.config().p());
}
