#include <google/gtest/gtest.h>
#include <messmer/cpp-utils/data/DataFixture.h>
#include <boost/optional/optional_io.hpp>
#include "../../../../src/config/crypto/outer/OuterConfig.h"

using cpputils::Data;
using cpputils::DataFixture;
using cpputils::DerivedKeyConfig;
using boost::none;
using std::ostream;
using namespace cryfs;

// This is needed for google test
namespace boost {
    ostream &operator<<(ostream &stream, const OuterConfig &) {
        return stream << "OuterConfig()";
    }
}

class OuterConfigTest: public ::testing::Test {
public:
    Data salt() {
        return DataFixture::generate(128, 2);
    }
    uint64_t N = 1024;
    uint8_t r = 1;
    uint8_t p = 2;
};

TEST_F(OuterConfigTest, SomeValues) {
    Data serialized = OuterConfig{DerivedKeyConfig(salt(), N, r, p), DataFixture::generate(1024)}.serialize();
    OuterConfig deserialized = OuterConfig::deserialize(serialized).value();
    EXPECT_EQ(DerivedKeyConfig(salt(), N, r, p), deserialized.keyConfig);
    EXPECT_EQ(DataFixture::generate(1024), deserialized.encryptedInnerConfig);
}

TEST_F(OuterConfigTest, DataEmpty) {
    Data serialized = OuterConfig{DerivedKeyConfig(salt(), N, r, p), Data(0)}.serialize();
    OuterConfig deserialized = OuterConfig::deserialize(serialized).value();
    EXPECT_EQ(DerivedKeyConfig(salt(), N, r, p), deserialized.keyConfig);
    EXPECT_EQ(Data(0), deserialized.encryptedInnerConfig);
}

TEST_F(OuterConfigTest, KeyConfigEmpty) {
    Data serialized = OuterConfig{DerivedKeyConfig(Data(0), 0, 0, 0), DataFixture::generate(1024)}.serialize();
    OuterConfig deserialized = OuterConfig::deserialize(serialized).value();
    EXPECT_EQ(DerivedKeyConfig(Data(0), 0, 0, 0), deserialized.keyConfig);
    EXPECT_EQ(DataFixture::generate(1024), deserialized.encryptedInnerConfig);
}

TEST_F(OuterConfigTest, DataAndKeyConfigEmpty) {
    Data serialized = OuterConfig{DerivedKeyConfig(Data(0), 0, 0, 0), Data(0)}.serialize();
    OuterConfig deserialized = OuterConfig::deserialize(serialized).value();
    EXPECT_EQ(DerivedKeyConfig(Data(0), 0, 0, 0), deserialized.keyConfig);
    EXPECT_EQ(Data(0), deserialized.encryptedInnerConfig);
}

TEST_F(OuterConfigTest, InvalidSerialization) {
    auto deserialized = OuterConfig::deserialize(DataFixture::generate(1024));
    EXPECT_EQ(none, deserialized);
}
