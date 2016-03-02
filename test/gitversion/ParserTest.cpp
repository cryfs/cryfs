#include <gtest/gtest.h>
#include <gitversion/parser.h>

using namespace gitversion;

TEST(ParserTest, TestUnknownVersion) {
    VersionInfo info = Parser::parse("0+unknown");
    EXPECT_EQ("0",   info.majorVersion);
    EXPECT_EQ("0",   info.minorVersion);
    EXPECT_TRUE(     info.isDevVersion);
    EXPECT_FALSE(    info.isStableVersion);
    EXPECT_EQ("",    info.gitCommitId);
    EXPECT_EQ("",    info.versionTag);
}

TEST(ParserTest, TestReleaseVersion_1) {
    VersionInfo info = Parser::parse("0.9.2");
    EXPECT_EQ("0",   info.majorVersion);
    EXPECT_EQ("9",   info.minorVersion);
    EXPECT_FALSE(    info.isDevVersion);
    EXPECT_TRUE(     info.isStableVersion);
    EXPECT_EQ("",    info.gitCommitId);
    EXPECT_EQ("",    info.versionTag);
}

TEST(ParserTest, TestReleaseVersion_2) {
    VersionInfo info = Parser::parse("1.02.3");
    EXPECT_EQ("1",   info.majorVersion);
    EXPECT_EQ("02",  info.minorVersion);
    EXPECT_FALSE(    info.isDevVersion);
    EXPECT_TRUE(     info.isStableVersion);
    EXPECT_EQ("",    info.gitCommitId);
    EXPECT_EQ("",    info.versionTag);
}

TEST(ParserTest, TestReleaseVersion_3) {
    VersionInfo info = Parser::parse("01.020.3");
    EXPECT_EQ("01",  info.majorVersion);
    EXPECT_EQ("020", info.minorVersion);
    EXPECT_FALSE(    info.isDevVersion);
    EXPECT_TRUE(     info.isStableVersion);
    EXPECT_EQ("",    info.gitCommitId);
    EXPECT_EQ("",    info.versionTag);
}

TEST(ParserTest, TestDirtyReleaseVersion) {
    VersionInfo info = Parser::parse("0.9.0+0.g5753e4f.dirty");
    EXPECT_EQ("0",   info.majorVersion);
    EXPECT_EQ("9",   info.minorVersion);
    EXPECT_TRUE(     info.isDevVersion);
    EXPECT_FALSE(    info.isStableVersion);
    EXPECT_EQ("5753e4f", info.gitCommitId);
    EXPECT_EQ("",    info.versionTag);
}


TEST(ParserTest, TestDevVersion) {
    VersionInfo info = Parser::parse("0.9.0+2.g0123abcdef");
    EXPECT_EQ("0",   info.majorVersion);
    EXPECT_EQ("9",   info.minorVersion);
    EXPECT_TRUE(     info.isDevVersion);
    EXPECT_FALSE(    info.isStableVersion);
    EXPECT_EQ("0123abcdef", info.gitCommitId);
    EXPECT_EQ("",    info.versionTag);
}

TEST(ParserTest, TestDirtyDevVersion) {
    VersionInfo info = Parser::parse("0.9.0+20.g0123abcdef.dirty");
    EXPECT_EQ("0",   info.majorVersion);
    EXPECT_EQ("9",   info.minorVersion);
    EXPECT_TRUE(     info.isDevVersion);
    EXPECT_FALSE(    info.isStableVersion);
    EXPECT_EQ("0123abcdef", info.gitCommitId);
    EXPECT_EQ("",    info.versionTag);
}

TEST(ParserTest, TestReleaseVersion_StableTag) {
    VersionInfo info = Parser::parse("0.9.2-stable");
    EXPECT_EQ("0",   info.majorVersion);
    EXPECT_EQ("9",   info.minorVersion);
    EXPECT_FALSE(    info.isDevVersion);
    EXPECT_TRUE(     info.isStableVersion);
    EXPECT_EQ("",    info.gitCommitId);
    EXPECT_EQ("stable",    info.versionTag);
}

TEST(ParserTest, TestDirtyReleaseVersion_StableTag) {
    VersionInfo info = Parser::parse("0.9.0-stable+0.g5753e4f.dirty");
    EXPECT_EQ("0",   info.majorVersion);
    EXPECT_EQ("9",   info.minorVersion);
    EXPECT_TRUE(     info.isDevVersion);
    EXPECT_FALSE(    info.isStableVersion);
    EXPECT_EQ("5753e4f", info.gitCommitId);
    EXPECT_EQ("stable",    info.versionTag);
}

TEST(ParserTest, TestDevVersion_StableTag) {
    VersionInfo info = Parser::parse("0.9.0-stable+2.g0123abcdef");
    EXPECT_EQ("0",   info.majorVersion);
    EXPECT_EQ("9",   info.minorVersion);
    EXPECT_TRUE(     info.isDevVersion);
    EXPECT_FALSE(    info.isStableVersion);
    EXPECT_EQ("0123abcdef", info.gitCommitId);
    EXPECT_EQ("stable",    info.versionTag);
}

TEST(ParserTest, TestDirtyDevVersion_StableTag) {
    VersionInfo info = Parser::parse("0.9.0-stable+20.g0123abcdef.dirty");
    EXPECT_EQ("0",   info.majorVersion);
    EXPECT_EQ("9",   info.minorVersion);
    EXPECT_TRUE(     info.isDevVersion);
    EXPECT_FALSE(    info.isStableVersion);
    EXPECT_EQ("0123abcdef", info.gitCommitId);
    EXPECT_EQ("stable",    info.versionTag);
}

TEST(ParserTest, TestReleaseVersion_AlphaTag) {
    VersionInfo info = Parser::parse("0.9.2-alpha");
    EXPECT_EQ("0",   info.majorVersion);
    EXPECT_EQ("9",   info.minorVersion);
    EXPECT_FALSE(    info.isDevVersion);
    EXPECT_FALSE(    info.isStableVersion);
    EXPECT_EQ("",    info.gitCommitId);
    EXPECT_EQ("alpha",    info.versionTag);
}

TEST(ParserTest, TestDirtyReleaseVersion_AlphaTag) {
    VersionInfo info = Parser::parse("0.9.0-alpha+0.g5753e4f.dirty");
    EXPECT_EQ("0",   info.majorVersion);
    EXPECT_EQ("9",   info.minorVersion);
    EXPECT_TRUE(     info.isDevVersion);
    EXPECT_FALSE(    info.isStableVersion);
    EXPECT_EQ("5753e4f", info.gitCommitId);
    EXPECT_EQ("alpha",    info.versionTag);
}

TEST(ParserTest, TestDevVersion_AlphaTag) {
    VersionInfo info = Parser::parse("0.9.0-alpha+2.g0123abcdef");
    EXPECT_EQ("0",   info.majorVersion);
    EXPECT_EQ("9",   info.minorVersion);
    EXPECT_TRUE(     info.isDevVersion);
    EXPECT_FALSE(    info.isStableVersion);
    EXPECT_EQ("0123abcdef", info.gitCommitId);
    EXPECT_EQ("alpha",    info.versionTag);
}

TEST(ParserTest, TestDirtyDevVersion_AlphaTag) {
    VersionInfo info = Parser::parse("0.9.0-alpha+20.g0123abcdef.dirty");
    EXPECT_EQ("0",   info.majorVersion);
    EXPECT_EQ("9",   info.minorVersion);
    EXPECT_TRUE(     info.isDevVersion);
    EXPECT_FALSE(    info.isStableVersion);
    EXPECT_EQ("0123abcdef", info.gitCommitId);
    EXPECT_EQ("alpha",    info.versionTag);
}

TEST(ParserTest, TestReleaseVersion_RCTag) {
    VersionInfo info = Parser::parse("0.9.2-rc1");
    EXPECT_EQ("0",   info.majorVersion);
    EXPECT_EQ("9",   info.minorVersion);
    EXPECT_FALSE(    info.isDevVersion);
    EXPECT_FALSE(    info.isStableVersion);
    EXPECT_EQ("",    info.gitCommitId);
    EXPECT_EQ("rc1",    info.versionTag);
}

TEST(ParserTest, TestDirtyReleaseVersion_RCTag) {
    VersionInfo info = Parser::parse("0.9.0-rc1+0.g5753e4f.dirty");
    EXPECT_EQ("0",   info.majorVersion);
    EXPECT_EQ("9",   info.minorVersion);
    EXPECT_TRUE(     info.isDevVersion);
    EXPECT_FALSE(    info.isStableVersion);
    EXPECT_EQ("5753e4f", info.gitCommitId);
    EXPECT_EQ("rc1",    info.versionTag);
}

TEST(ParserTest, TestDevVersion_RCTag) {
    VersionInfo info = Parser::parse("0.9.0-rc1+2.g0123abcdef");
    EXPECT_EQ("0",   info.majorVersion);
    EXPECT_EQ("9",   info.minorVersion);
    EXPECT_TRUE(     info.isDevVersion);
    EXPECT_FALSE(    info.isStableVersion);
    EXPECT_EQ("0123abcdef", info.gitCommitId);
    EXPECT_EQ("rc1",    info.versionTag);
}

TEST(ParserTest, TestDirtyDevVersion_RCTag) {
    VersionInfo info = Parser::parse("0.9.0-rc1+20.g0123abcdef.dirty");
    EXPECT_EQ("0",   info.majorVersion);
    EXPECT_EQ("9",   info.minorVersion);
    EXPECT_TRUE(     info.isDevVersion);
    EXPECT_FALSE(    info.isStableVersion);
    EXPECT_EQ("0123abcdef", info.gitCommitId);
    EXPECT_EQ("rc1",    info.versionTag);
}
