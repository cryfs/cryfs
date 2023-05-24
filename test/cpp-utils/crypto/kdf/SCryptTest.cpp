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

TEST_F(SCryptTest, BackwardsCompatibility_448) {
    SCrypt scrypt(SCrypt::TestSettings);
    auto kdfParameters = Data::FromString("00040000000000000100000002000000E429AFB0500BD5D172089598B76E6B9ED6D0DDAF3B08F99AA05357F96F4F7823");
    auto rederivedKey = scrypt.deriveExistingKey(56, "mypassword", kdfParameters);
    EXPECT_EQ("70416B4E1569E2335442F7FE740E6A8ADC149514B7B6D7838A996AE0E2125F743341E72FF9F44C91A9675EAE459C0C0126FDB6CE220436E0", rederivedKey.ToString());
}

TEST_F(SCryptTest, GeneratedKeyIsReproductible_256) {
    SCrypt scrypt(SCrypt::TestSettings);
    auto derivedKey = scrypt.deriveNewKey(32, "mypassword");
    auto rederivedKey = scrypt.deriveExistingKey(32, "mypassword", derivedKey.kdfParameters);
    EXPECT_TRUE(keyEquals(derivedKey.key, rederivedKey));
}

TEST_F(SCryptTest, BackwardsCompatibility_256) {
    SCrypt scrypt(SCrypt::TestSettings);
    auto kdfParameters = Data::FromString("000400000000000001000000020000007D65C035E0C4250003A24ED11ABD41F6101DEEC104F6875EE1B808A6683535BD");
    auto rederivedKey = scrypt.deriveExistingKey(32, "mypassword", kdfParameters);
    EXPECT_EQ("A423A0176F99A3197722D4B8686110FC2E2C04FF5E37AE43A7241097598F599D", rederivedKey.ToString());
}

TEST_F(SCryptTest, GeneratedKeyIsReproductible_128) {
    SCrypt scrypt(SCrypt::TestSettings);
    auto derivedKey = scrypt.deriveNewKey(16, "mypassword");
    auto rederivedKey = scrypt.deriveExistingKey(16, "mypassword", derivedKey.kdfParameters);
    EXPECT_TRUE(keyEquals(derivedKey.key, rederivedKey));
}

TEST_F(SCryptTest, BackwardsCompatibility_128) {
    SCrypt scrypt(SCrypt::TestSettings);
    auto kdfParameters = Data::FromString("000400000000000001000000020000008514339A7F583D80C9865C9EA01B698EE8AEAF99AE5F7AE79C8817D2E73D553D");
    auto rederivedKey = scrypt.deriveExistingKey(16, "mypassword", kdfParameters);
    EXPECT_EQ("2EF2F0A4EC335C961D4BE58BFB722F75", rederivedKey.ToString());
}

TEST_F(SCryptTest, GeneratedKeyIsReproductible_DefaultSettings) {
    SCrypt scrypt(SCrypt::DefaultSettings);
    auto derivedKey = scrypt.deriveNewKey(32, "mypassword");
    auto rederivedKey = scrypt.deriveExistingKey(32, "mypassword", derivedKey.kdfParameters);
    EXPECT_TRUE(keyEquals(derivedKey.key, rederivedKey));
}

TEST_F(SCryptTest, BackwardsCompatibility_DefaultSettings) {
    SCrypt scrypt(SCrypt::DefaultSettings);
    auto kdfParameters = Data::FromString("00001000000000000400000008000000D04ACF9519113E1F4E4D7FB39EFBF257CD71CF8536A468B546C2F5A65C6B622C");
    auto rederivedKey = scrypt.deriveExistingKey(32, "mypassword", kdfParameters);
    EXPECT_EQ("AB70B1923F3EB9EB8A75C15FD665AC3494C5EBAB80323D864135DBB2911ECF59", rederivedKey.ToString());
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
    EXPECT_EQ(SCrypt::TestSettings.N, parameters.n());
    EXPECT_EQ(SCrypt::TestSettings.r, parameters.r());
    EXPECT_EQ(SCrypt::TestSettings.p, parameters.p());
}

TEST_F(SCryptTest, UsesCorrectDefaultSettings) {
    SCrypt scrypt(SCrypt::DefaultSettings);
    auto derivedKey = scrypt.deriveNewKey(16, "mypassword");
    auto parameters = SCryptParameters::deserialize(derivedKey.kdfParameters);
    EXPECT_EQ(SCrypt::DefaultSettings.SALT_LEN, parameters.salt().size());
    EXPECT_EQ(SCrypt::DefaultSettings.N, parameters.n());
    EXPECT_EQ(SCrypt::DefaultSettings.r, parameters.r());
    EXPECT_EQ(SCrypt::DefaultSettings.p, parameters.p());
}
