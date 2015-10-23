#include <google/gtest/gtest.h>
#include "../../../src/config/crypto/Scrypt.h"

using namespace cryfs;

TEST(SCryptTest, GeneratedKeyIsReproductible_448) {
    auto created = SCrypt().generateKey<56>("mypassword");
    auto recreated = SCrypt().generateKeyFromConfig<56>("mypassword", created.config());
    EXPECT_EQ(created.key(), recreated);
}

TEST(SCryptTest, GeneratedKeyIsReproductible_256) {
    auto created = SCrypt().generateKey<32>("mypassword");
    auto recreated = SCrypt().generateKeyFromConfig<32>("mypassword", created.config());
    EXPECT_EQ(created.key(), recreated);
}

TEST(SCryptTest, GeneratedKeyIsReproductible_128) {
    auto created = SCrypt().generateKey<16>("mypassword");
    auto recreated = SCrypt().generateKeyFromConfig<16>("mypassword", created.config());
    EXPECT_EQ(created.key(), recreated);
}

TEST(SCryptTest, DifferentPasswordResultsInDifferentKey) {
    auto created = SCrypt().generateKey<16>("mypassword");
    auto recreated = SCrypt().generateKeyFromConfig<16>("mypassword2", created.config());
    EXPECT_NE(created.key(), recreated);
}

TEST(SCryptTest, UsesCorrectDefaultParameters) {
    auto created = SCrypt().generateKey<16>("mypassword");
    EXPECT_EQ(SCrypt::SALT_LEN, created.config().salt().size());
    EXPECT_EQ(SCrypt::N, created.config().N());
    EXPECT_EQ(SCrypt::r, created.config().r());
    EXPECT_EQ(SCrypt::p, created.config().p());
}
