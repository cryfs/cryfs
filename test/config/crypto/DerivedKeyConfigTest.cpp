#include <google/gtest/gtest.h>
#include "../../../src/config/crypto/DerivedKeyConfig.h"
#include <messmer/cpp-utils/data/DataFixture.h>

using namespace cryfs;
using cpputils::DataFixture;
using cpputils::Data;

TEST(DerivedKeyConfigTest, Salt) {
    DerivedKeyConfig cfg(DataFixture::generate(32), 0, 0, 0);
    EXPECT_EQ(DataFixture::generate(32), cfg.salt());
}

TEST(DerivedKeyConfigTest, Salt_Move) {
    DerivedKeyConfig cfg(DataFixture::generate(32), 0, 0, 0);
    DerivedKeyConfig moved = std::move(cfg);
    EXPECT_EQ(DataFixture::generate(32), moved.salt());
}

TEST(DerivedKeyConfigTest, N) {
    DerivedKeyConfig cfg(Data(0), 1024, 0, 0);
    EXPECT_EQ(1024, cfg.N());
}

TEST(DerivedKeyConfigTest, N_Move) {
    DerivedKeyConfig cfg(Data(0), 1024, 0, 0);
    DerivedKeyConfig moved = std::move(cfg);
    EXPECT_EQ(1024, moved.N());
}

TEST(DerivedKeyConfigTest, r) {
    DerivedKeyConfig cfg(Data(0), 0, 8, 0);
    EXPECT_EQ(8, cfg.r());
}

TEST(DerivedKeyConfigTest, r_Move) {
    DerivedKeyConfig cfg(Data(0), 0, 8, 0);
    DerivedKeyConfig moved = std::move(cfg);
    EXPECT_EQ(8, moved.r());
}

TEST(DerivedKeyConfigTest, p) {
    DerivedKeyConfig cfg(Data(0), 0, 0, 16);
    EXPECT_EQ(16, cfg.p());
}

TEST(DerivedKeyConfigTest, p_Move) {
    DerivedKeyConfig cfg(Data(0), 0, 0, 16);
    DerivedKeyConfig moved = std::move(cfg);
    EXPECT_EQ(16, moved.p());
}
