#include <google/gtest/gtest.h>
#include "../../../crypto/kdf/DerivedKeyConfig.h"
#include "../../../data/DataFixture.h"
#include <sstream>

using namespace cpputils;

class DerivedKeyConfigTest : public ::testing::Test {
public:
    DerivedKeyConfig SaveAndLoad(const DerivedKeyConfig &source) {
        Serializer serializer(source.serializedSize());
        source.serialize(&serializer);
        Data serialized = serializer.finished();
        Deserializer deserializer(&serialized);
        return DerivedKeyConfig::load(&deserializer);
    }
};

TEST_F(DerivedKeyConfigTest, Salt) {
    DerivedKeyConfig cfg(DataFixture::generate(32), 0, 0, 0);
    EXPECT_EQ(DataFixture::generate(32), cfg.salt());
}

TEST_F(DerivedKeyConfigTest, Salt_Move) {
    DerivedKeyConfig cfg(DataFixture::generate(32), 0, 0, 0);
    DerivedKeyConfig moved = std::move(cfg);
    EXPECT_EQ(DataFixture::generate(32), moved.salt());
}

TEST_F(DerivedKeyConfigTest, Salt_SaveAndLoad) {
    DerivedKeyConfig cfg(DataFixture::generate(32), 0, 0, 0);
    DerivedKeyConfig loaded = SaveAndLoad(cfg);
    EXPECT_EQ(DataFixture::generate(32), loaded.salt());
}

TEST_F(DerivedKeyConfigTest, N) {
    DerivedKeyConfig cfg(Data(0), 1024, 0, 0);
    EXPECT_EQ(1024, cfg.N());
}

TEST_F(DerivedKeyConfigTest, N_Move) {
    DerivedKeyConfig cfg(Data(0), 1024, 0, 0);
    DerivedKeyConfig moved = std::move(cfg);
    EXPECT_EQ(1024, moved.N());
}

TEST_F(DerivedKeyConfigTest, N_SaveAndLoad) {
    DerivedKeyConfig cfg(Data(0), 1024, 0, 0);
    DerivedKeyConfig loaded = SaveAndLoad(cfg);
    EXPECT_EQ(1024, loaded.N());
}

TEST_F(DerivedKeyConfigTest, r) {
    DerivedKeyConfig cfg(Data(0), 0, 8, 0);
    EXPECT_EQ(8, cfg.r());
}

TEST_F(DerivedKeyConfigTest, r_Move) {
    DerivedKeyConfig cfg(Data(0), 0, 8, 0);
    DerivedKeyConfig moved = std::move(cfg);
    EXPECT_EQ(8, moved.r());
}

TEST_F(DerivedKeyConfigTest, r_SaveAndLoad) {
    DerivedKeyConfig cfg(Data(0), 0, 8, 0);
    DerivedKeyConfig loaded = SaveAndLoad(cfg);
    EXPECT_EQ(8, loaded.r());
}

TEST_F(DerivedKeyConfigTest, p) {
    DerivedKeyConfig cfg(Data(0), 0, 0, 16);
    EXPECT_EQ(16, cfg.p());
}

TEST_F(DerivedKeyConfigTest, p_Move) {
    DerivedKeyConfig cfg(Data(0), 0, 0, 16);
    DerivedKeyConfig moved = std::move(cfg);
    EXPECT_EQ(16, moved.p());
}


TEST_F(DerivedKeyConfigTest, p_SaveAndLoad) {
    DerivedKeyConfig cfg(Data(0), 0, 0, 16);
    DerivedKeyConfig loaded = SaveAndLoad(cfg);
    EXPECT_EQ(16, loaded.p());
}