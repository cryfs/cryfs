#include <gtest/gtest.h>
#include "cpp-utils/crypto/kdf/Scrypt.h"

using namespace cpputils;
using std::string;

class SCryptTest : public ::testing::Test {
public:
    bool keyEquals(const EncryptionKey& lhs, const EncryptionKey& rhs) {
        ASSERT(lhs.binaryLength() == rhs.binaryLength(), "Keys must have equal size to be comparable");
        return 0 == std::memcmp(lhs.data(), rhs.data(), lhs.binaryLength());
    }
};

TEST_F(SCryptTest, GeneratedKeyIsReproductible_448) {
    SCrypt scrypt(SCrypt::TestSettings);
    auto derivedKey = scrypt.deriveNewKey(56, "mypassword");
    auto rederivedKey = scrypt.deriveExistingKey(56, "mypassword", derivedKey.kdfParameters);
    EXPECT_TRUE(keyEquals(derivedKey.key, rederivedKey));
}

TEST_F(SCryptTest, GeneratedKeyIsReproductible_256) {
    SCrypt scrypt(SCrypt::TestSettings);
    auto derivedKey = scrypt.deriveNewKey(32, "mypassword");
    auto rederivedKey = scrypt.deriveExistingKey(32, "mypassword", derivedKey.kdfParameters);
    EXPECT_TRUE(keyEquals(derivedKey.key, rederivedKey));
}

TEST_F(SCryptTest, GeneratedKeyIsReproductible_128) {
    SCrypt scrypt(SCrypt::TestSettings);
    auto derivedKey = scrypt.deriveNewKey(16, "mypassword");
    auto rederivedKey = scrypt.deriveExistingKey(16, "mypassword", derivedKey.kdfParameters);
    EXPECT_TRUE(keyEquals(derivedKey.key, rederivedKey));
}

TEST_F(SCryptTest, GeneratedKeyIsReproductible_DefaultSettings) {
    SCrypt scrypt(SCrypt::TestSettings);
    auto derivedKey = scrypt.deriveNewKey(16, "mypassword");
    auto rederivedKey = scrypt.deriveExistingKey(16, "mypassword", derivedKey.kdfParameters);
    EXPECT_TRUE(keyEquals(derivedKey.key, rederivedKey));
}

TEST_F(SCryptTest, DifferentPasswordResultsInDifferentKey) {
    SCrypt scrypt(SCrypt::TestSettings);
    auto derivedKey = scrypt.deriveNewKey(16, "mypassword");
    auto rederivedKey = scrypt.deriveExistingKey(16, "mypassword2", derivedKey.kdfParameters);
    EXPECT_FALSE(keyEquals(derivedKey.key, rederivedKey));
}

TEST_F(SCryptTest, UsesCorrectSettings) {
    SCrypt scrypt(SCrypt::TestSettings);
    auto derivedKey = scrypt.deriveNewKey(16, "mypassword");
    auto parameters = SCryptParameters::deserialize(derivedKey.kdfParameters);
    EXPECT_EQ(SCrypt::TestSettings.SALT_LEN, parameters.salt().size());
    EXPECT_EQ(SCrypt::TestSettings.N, parameters.N());
    EXPECT_EQ(SCrypt::TestSettings.r, parameters.r());
    EXPECT_EQ(SCrypt::TestSettings.p, parameters.p());
}

TEST_F(SCryptTest, UsesCorrectDefaultSettings) {
    SCrypt scrypt(SCrypt::DefaultSettings);
    auto derivedKey = scrypt.deriveNewKey(16, "mypassword");
    auto parameters = SCryptParameters::deserialize(derivedKey.kdfParameters);
    EXPECT_EQ(SCrypt::DefaultSettings.SALT_LEN, parameters.salt().size());
    EXPECT_EQ(SCrypt::DefaultSettings.N, parameters.N());
    EXPECT_EQ(SCrypt::DefaultSettings.r, parameters.r());
    EXPECT_EQ(SCrypt::DefaultSettings.p, parameters.p());
}
