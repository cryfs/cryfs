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
    ostream &operator<<(ostream &stream, const InnerConfig &config) {
        return stream << "InnerConfig(" << config.cipherName << ", [data])";
    }
}
#include <boost/optional/optional_io.hpp>

TEST(InnerConfigTest, SomeValues) {
    Data serialized = InnerConfig{"myciphername", DataFixture::generate(1024)}.serialize();
    InnerConfig deserialized = InnerConfig::deserialize(serialized).value();
    EXPECT_EQ("myciphername", deserialized.cipherName);
    EXPECT_EQ(DataFixture::generate(1024), deserialized.encryptedConfig);
}

TEST(InnerConfigTest, DataEmpty) {
    Data serialized = InnerConfig{"myciphername", Data(0)}.serialize();
    InnerConfig deserialized = InnerConfig::deserialize(serialized).value();
    EXPECT_EQ("myciphername", deserialized.cipherName);
    EXPECT_EQ(Data(0), deserialized.encryptedConfig);
}

TEST(InnerConfigTest, CipherNameEmpty) {
    Data serialized = InnerConfig{"", DataFixture::generate(1024)}.serialize();
    InnerConfig deserialized = InnerConfig::deserialize(serialized).value();
    EXPECT_EQ("", deserialized.cipherName);
    EXPECT_EQ(DataFixture::generate(1024), deserialized.encryptedConfig);
}

TEST(InnerConfigTest, DataAndCipherNameEmpty) {
    Data serialized = InnerConfig{"", Data(0)}.serialize();
    InnerConfig deserialized = InnerConfig::deserialize(serialized).value();
    EXPECT_EQ("", deserialized.cipherName);
    EXPECT_EQ(Data(0), deserialized.encryptedConfig);
}

TEST(InnerConfigTest, InvalidSerialization) {
    auto deserialized = InnerConfig::deserialize(DataFixture::generate(1024));
    EXPECT_EQ(none, deserialized);
}
