#include <gtest/gtest.h>
#include <cpp-utils/data/DataFixture.h>
#include <cryfs/impl/config/crypto/outer/OuterConfig.h>

using cpputils::Data;
using cpputils::DataFixture;
using boost::none;
using std::ostream;
using namespace cryfs;

// This is needed for google test
namespace boost {
    ostream &operator<<(ostream &stream, const OuterConfig &) {
        return stream << "OuterConfig()";
    }
}
#include <boost/optional/optional_io.hpp>

class OuterConfigTest: public ::testing::Test {
public:
    Data kdfParameters() {
        return DataFixture::generate(128, 2);
    }
};

TEST_F(OuterConfigTest, SomeValues) {
    Data serialized = OuterConfig{kdfParameters(), DataFixture::generate(1024), false}.serialize();
    OuterConfig deserialized = OuterConfig::deserialize(serialized).value();
    EXPECT_EQ(kdfParameters(), deserialized.kdfParameters);
    EXPECT_EQ(DataFixture::generate(1024), deserialized.encryptedInnerConfig);
}

TEST_F(OuterConfigTest, DataEmpty) {
    Data serialized = OuterConfig{kdfParameters(), Data(0), false}.serialize();
    OuterConfig deserialized = OuterConfig::deserialize(serialized).value();
    EXPECT_EQ(kdfParameters(), deserialized.kdfParameters);
    EXPECT_EQ(Data(0), deserialized.encryptedInnerConfig);
}

TEST_F(OuterConfigTest, KeyConfigEmpty) {
    Data serialized = OuterConfig{Data(0), DataFixture::generate(1024), false}.serialize();
    OuterConfig deserialized = OuterConfig::deserialize(serialized).value();
    EXPECT_EQ(Data(0), deserialized.kdfParameters);
    EXPECT_EQ(DataFixture::generate(1024), deserialized.encryptedInnerConfig);
}

TEST_F(OuterConfigTest, DataAndKeyConfigEmpty) {
    Data serialized = OuterConfig{Data(0), Data(0), false}.serialize();
    OuterConfig deserialized = OuterConfig::deserialize(serialized).value();
    EXPECT_EQ(Data(0), deserialized.kdfParameters);
    EXPECT_EQ(Data(0), deserialized.encryptedInnerConfig);
}

TEST_F(OuterConfigTest, InvalidSerialization) {
    auto deserialized = OuterConfig::deserialize(DataFixture::generate(1024));
    EXPECT_EQ(none, deserialized);
}
