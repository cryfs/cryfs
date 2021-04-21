#include <gtest/gtest.h>
#include <cpp-utils/crypto/kdf/SCryptParameters.h>
#include <cpp-utils/data/DataFixture.h>
#include <sstream>

using namespace cpputils;

class SCryptParametersTest : public ::testing::Test {
public:
    SCryptParameters SaveAndLoad(const SCryptParameters &source) {
        Data serialized = source.serialize();
        return SCryptParameters::deserialize(serialized);
    }
};

TEST_F(SCryptParametersTest, Salt) {
    SCryptParameters cfg(DataFixture::generate(32), 0, 0, 0);
    EXPECT_EQ(DataFixture::generate(32), cfg.salt());
}

TEST_F(SCryptParametersTest, Salt_Move) {
    SCryptParameters cfg(DataFixture::generate(32), 0, 0, 0);
    SCryptParameters moved = std::move(cfg);
    EXPECT_EQ(DataFixture::generate(32), moved.salt());
}

TEST_F(SCryptParametersTest, Salt_SaveAndLoad) {
    SCryptParameters cfg(DataFixture::generate(32), 0, 0, 0);
    SCryptParameters loaded = SaveAndLoad(cfg);
    EXPECT_EQ(DataFixture::generate(32), loaded.salt());
}

TEST_F(SCryptParametersTest, N) {
    SCryptParameters cfg(Data(0), 1024, 0, 0);
    EXPECT_EQ(1024u, cfg.n());
}

TEST_F(SCryptParametersTest, N_Move) {
    SCryptParameters cfg(Data(0), 1024, 0, 0);
    SCryptParameters moved = std::move(cfg);
    EXPECT_EQ(1024u, moved.n());
}

TEST_F(SCryptParametersTest, N_SaveAndLoad) {
    SCryptParameters cfg(Data(0), 1024, 0, 0);
    SCryptParameters loaded = SaveAndLoad(cfg);
    EXPECT_EQ(1024u, loaded.n());
}

TEST_F(SCryptParametersTest, r) {
    SCryptParameters cfg(Data(0), 0, 8, 0);
    EXPECT_EQ(8u, cfg.r());
}

TEST_F(SCryptParametersTest, r_Move) {
    SCryptParameters cfg(Data(0), 0, 8, 0);
    SCryptParameters moved = std::move(cfg);
    EXPECT_EQ(8u, moved.r());
}

TEST_F(SCryptParametersTest, r_SaveAndLoad) {
    SCryptParameters cfg(Data(0), 0, 8, 0);
    SCryptParameters loaded = SaveAndLoad(cfg);
    EXPECT_EQ(8u, loaded.r());
}

TEST_F(SCryptParametersTest, p) {
    SCryptParameters cfg(Data(0), 0, 0, 16);
    EXPECT_EQ(16u, cfg.p());
}

TEST_F(SCryptParametersTest, p_Move) {
    SCryptParameters cfg(Data(0), 0, 0, 16);
    SCryptParameters moved = std::move(cfg);
    EXPECT_EQ(16u, moved.p());
}


TEST_F(SCryptParametersTest, p_SaveAndLoad) {
    SCryptParameters cfg(Data(0), 0, 0, 16);
    SCryptParameters loaded = SaveAndLoad(cfg);
    EXPECT_EQ(16u, loaded.p());
}
