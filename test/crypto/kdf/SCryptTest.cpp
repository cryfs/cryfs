#include <google/gtest/gtest.h>
#include "../../../crypto/kdf/Scrypt.h"
#include "testutils/SCryptTestSettings.h"

using namespace cpputils;

TEST(SCryptTest, GeneratedKeyIsReproductible_448) {
    auto created = SCrypt().generateKey<56, SCryptTestSettings>("mypassword");
    auto recreated = SCrypt().generateKeyFromConfig<56>("mypassword", created.config());
    EXPECT_EQ(created.key(), recreated);
}

TEST(SCryptTest, GeneratedKeyIsReproductible_256) {
    auto created = SCrypt().generateKey<32, SCryptTestSettings>("mypassword");
    auto recreated = SCrypt().generateKeyFromConfig<32>("mypassword", created.config());
    EXPECT_EQ(created.key(), recreated);
}

TEST(SCryptTest, GeneratedKeyIsReproductible_128) {
    auto created = SCrypt().generateKey<16, SCryptTestSettings>("mypassword");
    auto recreated = SCrypt().generateKeyFromConfig<16>("mypassword", created.config());
    EXPECT_EQ(created.key(), recreated);
}

TEST(SCryptTest, GeneratedKeyIsReproductible_DefaultSettings) {
    auto created = SCrypt().generateKey<16>("mypassword");
    auto recreated = SCrypt().generateKeyFromConfig<16>("mypassword", created.config());
    EXPECT_EQ(created.key(), recreated);
}

TEST(SCryptTest, DifferentPasswordResultsInDifferentKey) {
    auto created = SCrypt().generateKey<16, SCryptTestSettings>("mypassword");
    auto recreated = SCrypt().generateKeyFromConfig<16>("mypassword2", created.config());
    EXPECT_NE(created.key(), recreated);
}

TEST(SCryptTest, UsesCorrectSettings) {
    auto created = SCrypt().generateKey<16, SCryptTestSettings>("mypassword");
    EXPECT_EQ(SCryptTestSettings::SALT_LEN, created.config().salt().size());
    EXPECT_EQ(SCryptTestSettings::N, created.config().N());
    EXPECT_EQ(SCryptTestSettings::r, created.config().r());
    EXPECT_EQ(SCryptTestSettings::p, created.config().p());
}

TEST(SCryptTest, UsesCorrectDefaultSettings) {
    auto created = SCrypt().generateKey<16>("mypassword");
    EXPECT_EQ(SCryptDefaultSettings::SALT_LEN, created.config().salt().size());
    EXPECT_EQ(SCryptDefaultSettings::N, created.config().N());
    EXPECT_EQ(SCryptDefaultSettings::r, created.config().r());
    EXPECT_EQ(SCryptDefaultSettings::p, created.config().p());
}
