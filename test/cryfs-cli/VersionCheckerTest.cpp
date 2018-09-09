#include <gtest/gtest.h>
#include <cryfs-cli/VersionChecker.h>
#include <cpp-utils/network/FakeHttpClient.h>
#include <cpp-utils/pointer/unique_ref.h>

using std::string;
using cpputils::FakeHttpClient;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::none;
using namespace cryfs;

class VersionCheckerTest: public ::testing::Test {
public:
    unique_ref<VersionChecker> versionChecker() {
        return make_unique_ref<VersionChecker>(_http.get());
    }

    void setVersionInfo(const string &versionInfo) {
        _http->addWebsite("https://www.cryfs.org/version_info.json", versionInfo);
    }

private:
    unique_ref<FakeHttpClient> _http = make_unique_ref<FakeHttpClient>();
};

TEST_F(VersionCheckerTest, NewestVersion_NoInternet) {
    EXPECT_EQ(none, versionChecker()->newestVersion());
}

TEST_F(VersionCheckerTest, SecurityWarningFor_NoInternet) {
    EXPECT_EQ(none, versionChecker()->securityWarningFor("0.8"));
}

TEST_F(VersionCheckerTest, NewestVersion_NoWarnings_1) {
    setVersionInfo("{\"version_info\":{\"current\":\"0.8.2\"}}");
    EXPECT_EQ("0.8.2", versionChecker()->newestVersion().value());
}

TEST_F(VersionCheckerTest, NewestVersion_NoWarnings_2) {
    setVersionInfo("{\"version_info\":{\"current\":\"0.8.3\"}}");
    EXPECT_EQ("0.8.3", versionChecker()->newestVersion().value());
}

TEST_F(VersionCheckerTest, NewestVersion_EmptyWarnings) {
    setVersionInfo("{\"version_info\":{\"current\":\"0.8.2\"},\"warnings\":{}}");
    EXPECT_EQ("0.8.2", versionChecker()->newestVersion().value());
}

TEST_F(VersionCheckerTest, NewestVersion_WarningsOtherVersion) {
    setVersionInfo("{\"version_info\":{\"current\":\"0.8.2\"},\"warnings\":{\"0.8.1\": \"warning\"}}");
    EXPECT_EQ("0.8.2", versionChecker()->newestVersion().value());
}

TEST_F(VersionCheckerTest, NewestVersion_WarningsSameVersion) {
    setVersionInfo("{\"version_info\":{\"current\":\"0.8.2\"},\"warnings\":{\"0.8.2\": \"warning\"}}");
    EXPECT_EQ("0.8.2", versionChecker()->newestVersion().value());
}

TEST_F(VersionCheckerTest, NewestVersion_WarningsSameAndOtherVersion) {
    setVersionInfo("{\"version_info\":{\"current\":\"0.8.2\"},\"warnings\":{\"0.8.1\": \"warning1\", \"0.8.2\": \"warning2\", \"0.8.3\": \"warning3\"}}");
    EXPECT_EQ("0.8.2", versionChecker()->newestVersion().value());
}

TEST_F(VersionCheckerTest, NewestVersion_BlankVersionInfo) {
    setVersionInfo("");
    EXPECT_EQ(none, versionChecker()->newestVersion());
}

TEST_F(VersionCheckerTest, NewestVersion_EmptyVersionInfo) {
    setVersionInfo("{}");
    EXPECT_EQ(none, versionChecker()->newestVersion());
}

TEST_F(VersionCheckerTest, NewestVersion_InvalidVersionInfo) {
    setVersionInfo("invalid-json");
    EXPECT_EQ(none, versionChecker()->newestVersion());
}

TEST_F(VersionCheckerTest, NewestVersion_MissingKey) {
    setVersionInfo("{\"warnings\":{}");
    EXPECT_EQ(none, versionChecker()->newestVersion());
}

TEST_F(VersionCheckerTest, SecurityWarningFor_NoWarnings) {
    setVersionInfo("{\"version_info\":{\"current\":\"0.8.2\"}}");
    EXPECT_EQ(none, versionChecker()->securityWarningFor("0.8.2"));
}

TEST_F(VersionCheckerTest, SecurityWarningFor_EmptyWarnings) {
    setVersionInfo("{\"version_info\":{\"current\":\"0.8.2\"},\"warnings\":{}}");
    EXPECT_EQ(none, versionChecker()->securityWarningFor("0.8.2"));
}

TEST_F(VersionCheckerTest, SecurityWarningFor_WarningsOtherVersion) {
    setVersionInfo("{\"version_info\":{\"current\":\"0.8.2\"},\"warnings\":{\"0.8.1\": \"warning\"}}");
    EXPECT_EQ(none, versionChecker()->securityWarningFor("0.8.2"));
}

TEST_F(VersionCheckerTest, SecurityWarningFor_WarningsSameVersion) {
    setVersionInfo("{\"version_info\":{\"current\":\"0.8.2\"},\"warnings\":{\"0.8.2\": \"warning\"}}");
    EXPECT_EQ("warning", versionChecker()->securityWarningFor("0.8.2").value());
}

TEST_F(VersionCheckerTest, SecurityWarningFor_WarningsSameAndOtherVersion) {
    setVersionInfo("{\"version_info\":{\"current\":\"0.8.2\"},\"warnings\":{\"0.8.1\": \"warning1\", \"0.8.2\": \"warning2\", \"0.8.3\": \"warning3\"}}");
    EXPECT_EQ("warning2", versionChecker()->securityWarningFor("0.8.2").value());
}

TEST_F(VersionCheckerTest, SecurityWarningFor_BlankVersionInfo) {
    setVersionInfo("");
    EXPECT_EQ(none, versionChecker()->securityWarningFor("0.8.2"));
}

TEST_F(VersionCheckerTest, SecurityWarningFor_EmptyVersionInfo) {
    setVersionInfo("{}");
    EXPECT_EQ(none, versionChecker()->securityWarningFor("0.8.2"));
}

TEST_F(VersionCheckerTest, SecurityWarningFor_InvalidVersionInfo) {
    setVersionInfo("invalid-json");
    EXPECT_EQ(none, versionChecker()->securityWarningFor("0.8.2"));
}
