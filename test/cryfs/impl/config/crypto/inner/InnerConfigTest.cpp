#include <gtest/gtest.h>
#include <cpp-utils/data/DataFixture.h>
#include <cryfs/impl/config/crypto/inner/InnerConfig.h>

using cpputils::Data;
using cpputils::DataFixture;
using boost::none;
using std::ostream;
using namespace cryfs;

// This is needed for google test
namespace boost {
    // gtest finds this printer by ADL, which looks in the enclosing namespace itself and
    // does not follow the implicit using-directive of a nested unnamed namespace. Moving it
    // into one would silently stop it being found, so static is the right mechanism here.
    static ostream &operator<<(ostream &stream, const InnerConfig &config) {  // NOLINT(misc-use-anonymous-namespace)
        return stream << "InnerConfig(" << config.cipherName << ", [data])";
    }
}
#include <boost/optional/optional_io.hpp>

TEST(InnerConfigTest, SomeValues) {
    const Data serialized = InnerConfig{"myciphername", DataFixture::generate(1024)}.serialize();
    const InnerConfig deserialized = InnerConfig::deserialize(serialized).value();
    EXPECT_EQ("myciphername", deserialized.cipherName);
    EXPECT_EQ(DataFixture::generate(1024), deserialized.encryptedConfig);
}

TEST(InnerConfigTest, DataEmpty) {
    const Data serialized = InnerConfig{"myciphername", Data(0)}.serialize();
    const InnerConfig deserialized = InnerConfig::deserialize(serialized).value();
    EXPECT_EQ("myciphername", deserialized.cipherName);
    EXPECT_EQ(Data(0), deserialized.encryptedConfig);
}

TEST(InnerConfigTest, CipherNameEmpty) {
    const Data serialized = InnerConfig{"", DataFixture::generate(1024)}.serialize();
    const InnerConfig deserialized = InnerConfig::deserialize(serialized).value();
    EXPECT_EQ("", deserialized.cipherName);
    EXPECT_EQ(DataFixture::generate(1024), deserialized.encryptedConfig);
}

TEST(InnerConfigTest, DataAndCipherNameEmpty) {
    const Data serialized = InnerConfig{"", Data(0)}.serialize();
    const InnerConfig deserialized = InnerConfig::deserialize(serialized).value();
    EXPECT_EQ("", deserialized.cipherName);
    EXPECT_EQ(Data(0), deserialized.encryptedConfig);
}

TEST(InnerConfigTest, InvalidSerialization) {
    auto deserialized = InnerConfig::deserialize(DataFixture::generate(1024));
    EXPECT_EQ(none, deserialized);
}
