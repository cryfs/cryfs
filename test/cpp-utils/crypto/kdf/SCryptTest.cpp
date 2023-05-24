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
    auto kdfParameters = Data::FromString("000400000000000001000000010000006525D6D08CF88ADD10180F90322372A04F9E1C6D98264D94A7AC5CADF6286F23");
    auto rederivedKey = scrypt.deriveExistingKey(56, "mypassword", kdfParameters);
    EXPECT_EQ("460C501CC3BD2A26C2C82ABF72DB07616793F6B0EBAE5AD4E00BE1813B154BC9F22E58D9B49B123CC0D354A7DBF7BEC7325F3838455E932B", rederivedKey.ToString());
}

TEST_F(SCryptTest, GeneratedKeyIsReproductible_256) {
    SCrypt scrypt(SCrypt::TestSettings);
    auto derivedKey = scrypt.deriveNewKey(32, "mypassword");
    auto rederivedKey = scrypt.deriveExistingKey(32, "mypassword", derivedKey.kdfParameters);
    EXPECT_TRUE(keyEquals(derivedKey.key, rederivedKey));
}

TEST_F(SCryptTest, BackwardsCompatibility_256) {
    SCrypt scrypt(SCrypt::TestSettings);
    auto kdfParameters = Data::FromString("000400000000000001000000010000008E1E5C8BE2665897FCFA96E829CB3322824B174F295382673D43AF752AC51447");
    auto rederivedKey = scrypt.deriveExistingKey(32, "mypassword", kdfParameters);
    EXPECT_EQ("00C193FB9028F1371590FB9309F254377FFC3B6E1DDBBD5E0AD2F56AE1900D91", rederivedKey.ToString());
}

TEST_F(SCryptTest, GeneratedKeyIsReproductible_128) {
    SCrypt scrypt(SCrypt::TestSettings);
    auto derivedKey = scrypt.deriveNewKey(16, "mypassword");
    auto rederivedKey = scrypt.deriveExistingKey(16, "mypassword", derivedKey.kdfParameters);
    EXPECT_TRUE(keyEquals(derivedKey.key, rederivedKey));
}

TEST_F(SCryptTest, BackwardsCompatibility_128) {
    SCrypt scrypt(SCrypt::TestSettings);
    auto kdfParameters = Data::FromString("00040000000000000100000001000000C66B1F2B1175C23909488AB895A4E8BFCF59A4878AED5B299C37E445820EB415");
    auto rederivedKey = scrypt.deriveExistingKey(16, "mypassword", kdfParameters);
    EXPECT_EQ("24F0FB654ECB04827B1621CF4E00858F", rederivedKey.ToString());
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
